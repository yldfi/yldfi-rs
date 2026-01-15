//! Log decoding using ABI or event signatures

use crate::abi::EventSignature;
use crate::error::{AbiError, Result};
use alloy::dyn_abi::{DynSolType, DynSolValue};
use alloy::json_abi::{Event, JsonAbi};
use alloy::primitives::{Address, B256};
use alloy::rpc::types::Log;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A decoded log with named parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedLog {
    /// Block number
    pub block_number: u64,
    /// Block timestamp (Unix seconds, if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    /// Transaction hash
    pub transaction_hash: B256,
    /// Log index within the block
    pub log_index: u64,
    /// Contract address
    pub address: Address,
    /// Event name
    pub event_name: String,
    /// Event signature (canonical)
    pub event_signature: String,
    /// Decoded parameters
    pub params: HashMap<String, DecodedValue>,
    /// Raw topics (for reference)
    pub topics: Vec<B256>,
    /// Raw data (for reference)
    pub data: Vec<u8>,
}

/// A decoded parameter value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DecodedValue {
    Address(String),
    Uint(String),
    Int(String),
    Bool(bool),
    Bytes(String),
    String(String),
    Array(Vec<DecodedValue>),
    Tuple(Vec<DecodedValue>),
}

impl DecodedValue {
    /// Convert from DynSolValue
    fn from_dyn_sol_value(value: &DynSolValue) -> Self {
        match value {
            DynSolValue::Address(addr) => DecodedValue::Address(format!("{:#x}", addr)),
            DynSolValue::Bool(b) => DecodedValue::Bool(*b),
            DynSolValue::Bytes(b) => DecodedValue::Bytes(format!("0x{}", hex::encode(b))),
            DynSolValue::FixedBytes(b, _) => {
                DecodedValue::Bytes(format!("0x{}", hex::encode(b.as_slice())))
            }
            DynSolValue::Int(i, _) => DecodedValue::Int(i.to_string()),
            DynSolValue::Uint(u, _) => DecodedValue::Uint(u.to_string()),
            DynSolValue::String(s) => DecodedValue::String(s.clone()),
            DynSolValue::Array(arr) => {
                DecodedValue::Array(arr.iter().map(Self::from_dyn_sol_value).collect())
            }
            DynSolValue::FixedArray(arr) => {
                DecodedValue::Array(arr.iter().map(Self::from_dyn_sol_value).collect())
            }
            DynSolValue::Tuple(arr) => {
                DecodedValue::Tuple(arr.iter().map(Self::from_dyn_sol_value).collect())
            }
            _ => DecodedValue::String(format!("{:?}", value)),
        }
    }
}

/// Log decoder using ABI or event signatures
pub struct LogDecoder {
    /// Events indexed by topic0
    events: HashMap<B256, EventInfo>,
}

/// Event info for decoding
struct EventInfo {
    name: String,
    canonical: String,
    indexed_types: Vec<DynSolType>,
    data_types: Vec<DynSolType>,
    indexed_names: Vec<String>,
    data_names: Vec<String>,
    /// All param types in order (for dynamic re-partitioning when indexed isn't specified)
    all_types: Vec<DynSolType>,
    /// All param names in order
    all_names: Vec<String>,
    /// Whether indexed was explicitly specified (vs inferred)
    indexed_explicit: bool,
}

impl Default for LogDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl LogDecoder {
    /// Create an empty decoder
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    /// Create a new decoder from an ABI
    pub fn from_abi(abi: &JsonAbi) -> Result<Self> {
        let mut events = HashMap::new();

        for event in abi.events() {
            let info = Self::event_to_info(event)?;
            events.insert(event.selector(), info);
        }

        Ok(Self { events })
    }

    /// Create a decoder from event signatures
    pub fn from_signatures(signatures: &[EventSignature]) -> Result<Self> {
        let mut events = HashMap::new();

        for sig in signatures {
            let info = Self::signature_to_info(sig)?;
            events.insert(sig.topic, info);
        }

        Ok(Self { events })
    }

    /// Create a decoder for a single event signature
    pub fn from_signature(signature: &EventSignature) -> Result<Self> {
        Self::from_signatures(std::slice::from_ref(signature))
    }

    /// Add an event signature to the decoder
    pub fn add_signature(&mut self, signature: &EventSignature) -> Result<()> {
        let info = Self::signature_to_info(signature)?;
        self.events.insert(signature.topic, info);
        Ok(())
    }

    /// Convert an ABI Event to EventInfo
    fn event_to_info(event: &Event) -> Result<EventInfo> {
        let mut indexed_types = Vec::new();
        let mut indexed_names = Vec::new();
        let mut data_types = Vec::new();
        let mut data_names = Vec::new();
        let mut all_types = Vec::new();
        let mut all_names = Vec::new();

        for input in &event.inputs {
            let ty = Self::resolve_event_param_type(input)?;
            all_types.push(ty.clone());
            all_names.push(input.name.clone());

            if input.indexed {
                indexed_names.push(input.name.clone());
                indexed_types.push(ty);
            } else {
                data_names.push(input.name.clone());
                data_types.push(ty);
            }
        }

        // Build canonical signature using resolved types
        let param_types: Vec<String> = event
            .inputs
            .iter()
            .map(Self::canonical_type_string)
            .collect();
        let canonical = format!("{}({})", event.name, param_types.join(","));

        Ok(EventInfo {
            name: event.name.clone(),
            canonical,
            indexed_types,
            data_types,
            indexed_names,
            data_names,
            all_types,
            all_names,
            indexed_explicit: true, // ABI always has explicit indexed info
        })
    }

    /// Resolve an EventParam type, handling tuples with components
    fn resolve_event_param_type(param: &alloy::json_abi::EventParam) -> Result<DynSolType> {
        use alloy::json_abi::Param;

        // Helper to resolve Param types recursively
        fn resolve_param_type(param: &Param) -> Result<DynSolType> {
            let ty_str = param.ty.as_str();

            // Check if this is a tuple type (has components)
            if !param.components.is_empty() {
                // Build tuple type from components
                let mut component_types = Vec::new();
                for comp in &param.components {
                    component_types.push(resolve_param_type(comp)?);
                }
                let tuple_type = DynSolType::Tuple(component_types);

                // Check if it's an array of tuples
                if ty_str.ends_with("[]") {
                    Ok(DynSolType::Array(Box::new(tuple_type)))
                } else if ty_str.contains('[') {
                    // Fixed size array: tuple[N]
                    if let Some(start) = ty_str.rfind('[') {
                        if let Some(end) = ty_str.rfind(']') {
                            let size_str = &ty_str[start + 1..end];
                            if let Ok(size) = size_str.parse::<usize>() {
                                return Ok(DynSolType::FixedArray(Box::new(tuple_type), size));
                            }
                        }
                    }
                    // Fallback to dynamic array if parsing fails
                    Ok(DynSolType::Array(Box::new(tuple_type)))
                } else {
                    Ok(tuple_type)
                }
            } else {
                // Not a tuple, parse directly
                DynSolType::parse(ty_str).map_err(|e| {
                    AbiError::ParseError(format!("Invalid type '{}': {}", ty_str, e)).into()
                })
            }
        }

        // Convert EventParam to Param-like structure for recursive resolution
        let ty_str = param.ty.as_str();

        if !param.components.is_empty() {
            // Build tuple type from components
            let mut component_types = Vec::new();
            for comp in &param.components {
                component_types.push(resolve_param_type(comp)?);
            }
            let tuple_type = DynSolType::Tuple(component_types);

            // Check if it's an array of tuples
            if ty_str.ends_with("[]") {
                Ok(DynSolType::Array(Box::new(tuple_type)))
            } else if ty_str.contains('[') {
                // Fixed size array: tuple[N]
                if let Some(start) = ty_str.rfind('[') {
                    if let Some(end) = ty_str.rfind(']') {
                        let size_str = &ty_str[start + 1..end];
                        if let Ok(size) = size_str.parse::<usize>() {
                            return Ok(DynSolType::FixedArray(Box::new(tuple_type), size));
                        }
                    }
                }
                // Fallback to dynamic array if parsing fails
                Ok(DynSolType::Array(Box::new(tuple_type)))
            } else {
                Ok(tuple_type)
            }
        } else {
            // Not a tuple, parse directly
            Self::parse_type(ty_str)
        }
    }

    /// Build canonical type string for an EventParam (for event signature)
    fn canonical_type_string(param: &alloy::json_abi::EventParam) -> String {
        use alloy::json_abi::Param;

        fn param_canonical(param: &Param) -> String {
            if !param.components.is_empty() {
                // Tuple: build (type1,type2,...)
                let inner: Vec<String> = param.components.iter().map(param_canonical).collect();
                let tuple_str = format!("({})", inner.join(","));

                // Handle array suffix
                let ty = param.ty.as_str();
                if let Some(bracket_pos) = ty.find('[') {
                    format!("{}{}", tuple_str, &ty[bracket_pos..])
                } else {
                    tuple_str
                }
            } else {
                param.ty.clone()
            }
        }

        if !param.components.is_empty() {
            let inner: Vec<String> = param.components.iter().map(param_canonical).collect();
            let tuple_str = format!("({})", inner.join(","));

            let ty = param.ty.as_str();
            if let Some(bracket_pos) = ty.find('[') {
                format!("{}{}", tuple_str, &ty[bracket_pos..])
            } else {
                tuple_str
            }
        } else {
            param.ty.clone()
        }
    }

    /// Convert an EventSignature to EventInfo
    fn signature_to_info(sig: &EventSignature) -> Result<EventInfo> {
        let mut indexed_types = Vec::new();
        let mut indexed_names = Vec::new();
        let mut data_types = Vec::new();
        let mut data_names = Vec::new();
        let mut all_types = Vec::new();
        let mut all_names = Vec::new();
        let mut has_any_indexed = false;

        for (i, param) in sig.params.iter().enumerate() {
            let ty = Self::parse_type(&param.ty)?;
            let name = if param.name.is_empty() {
                format!("param{}", i)
            } else {
                param.name.clone()
            };

            all_types.push(ty.clone());
            all_names.push(name.clone());

            if param.indexed {
                has_any_indexed = true;
                indexed_names.push(name);
                indexed_types.push(ty);
            } else {
                data_names.push(name);
                data_types.push(ty);
            }
        }

        Ok(EventInfo {
            name: sig.name.clone(),
            canonical: sig.canonical.clone(),
            indexed_types,
            data_types,
            indexed_names,
            data_names,
            all_types,
            all_names,
            indexed_explicit: has_any_indexed, // Only explicit if user specified "indexed"
        })
    }

    /// Parse a Solidity type string into DynSolType
    fn parse_type(ty: &str) -> Result<DynSolType> {
        DynSolType::parse(ty)
            .map_err(|e| AbiError::ParseError(format!("Invalid type '{}': {}", ty, e)).into())
    }

    /// Decode a log
    pub fn decode(&self, log: &Log) -> Result<DecodedLog> {
        // Get topic0 (event selector)
        let topic0 = log
            .topics()
            .first()
            .ok_or_else(|| AbiError::DecodeError("Log has no topics".to_string()))?;

        // Find event info
        let event_info = self
            .events
            .get(topic0)
            .ok_or_else(|| AbiError::EventNotFound(format!("Unknown event: {:#x}", topic0)))?;

        // Number of indexed params (topics excluding topic0)
        let indexed_topics: Vec<_> = log.topics().iter().skip(1).collect();
        let actual_indexed_count = indexed_topics.len();

        // Determine indexed/data split
        // If indexed was explicitly specified, use the stored split
        // Otherwise, infer from the log's topic count
        let (indexed_types, indexed_names, data_types, data_names): (
            Vec<DynSolType>,
            Vec<String>,
            Vec<DynSolType>,
            Vec<String>,
        ) = if event_info.indexed_explicit {
            // Use the explicitly specified split
            (
                event_info.indexed_types.clone(),
                event_info.indexed_names.clone(),
                event_info.data_types.clone(),
                event_info.data_names.clone(),
            )
        } else {
            // Infer indexed from log's topic count
            // First N params are indexed, rest are data
            let n = actual_indexed_count.min(event_info.all_types.len());
            (
                event_info.all_types[..n].to_vec(),
                event_info.all_names[..n].to_vec(),
                event_info.all_types[n..].to_vec(),
                event_info.all_names[n..].to_vec(),
            )
        };

        let mut params = HashMap::new();

        // Decode indexed parameters (topics[1..])
        for (i, (ty, name)) in indexed_types.iter().zip(indexed_names.iter()).enumerate() {
            if let Some(topic) = indexed_topics.get(i) {
                let value = Self::decode_indexed(ty, topic)?;
                params.insert(name.clone(), value);
            }
        }

        // Decode non-indexed parameters (data)
        if !data_types.is_empty() {
            if log.data().data.is_empty() {
                // Event expects data but none provided - log warning and use empty values
                tracing::warn!(
                    "Event '{}' expects {} non-indexed parameters but log data is empty \
                     (block {:?}, tx {:?}). Using empty placeholder values.",
                    event_info.name,
                    data_types.len(),
                    log.block_number,
                    log.transaction_hash
                );
                // Insert placeholder empty values for expected params
                for name in &data_names {
                    params.insert(name.clone(), DecodedValue::String("".to_string()));
                }
            } else {
                let decoded = Self::decode_data(&data_types, &log.data().data)?;

                for (name, value) in data_names.iter().zip(decoded.into_iter()) {
                    params.insert(name.clone(), value);
                }
            }
        }

        Ok(DecodedLog {
            block_number: log.block_number.unwrap_or(0),
            timestamp: None, // Filled in later if --timestamps is used
            transaction_hash: log.transaction_hash.unwrap_or_default(),
            log_index: log.log_index.unwrap_or(0),
            address: log.address(),
            event_name: event_info.name.clone(),
            event_signature: event_info.canonical.clone(),
            params,
            topics: log.topics().to_vec(),
            data: log.data().data.to_vec(),
        })
    }

    /// Decode an indexed parameter from a topic
    fn decode_indexed(ty: &DynSolType, topic: &B256) -> Result<DecodedValue> {
        // For dynamic types (string, bytes, arrays), the topic contains a hash
        match ty {
            DynSolType::String | DynSolType::Bytes | DynSolType::Array(_) => {
                // Cannot decode - return the hash
                Ok(DecodedValue::Bytes(format!("{:#x}", topic)))
            }
            _ => {
                // Decode from 32-byte topic
                let decoded = ty.abi_decode(&topic.0).map_err(|e| {
                    AbiError::DecodeError(format!("Failed to decode indexed param: {}", e))
                })?;
                Ok(DecodedValue::from_dyn_sol_value(&decoded))
            }
        }
    }

    /// Decode non-indexed parameters from data
    fn decode_data(types: &[DynSolType], data: &[u8]) -> Result<Vec<DecodedValue>> {
        let tuple_type = DynSolType::Tuple(types.to_vec());
        let decoded = tuple_type
            .abi_decode(data)
            .map_err(|e| AbiError::DecodeError(format!("Failed to decode data: {}", e)))?;

        match decoded {
            DynSolValue::Tuple(values) => Ok(values
                .iter()
                .map(DecodedValue::from_dyn_sol_value)
                .collect()),
            _ => Err(AbiError::DecodeError("Expected tuple".to_string()).into()),
        }
    }

    /// Check if a log can be decoded by this decoder
    pub fn can_decode(&self, log: &Log) -> bool {
        log.topics()
            .first()
            .map(|t| self.events.contains_key(t))
            .unwrap_or(false)
    }

    /// Get list of event names this decoder handles
    pub fn event_names(&self) -> Vec<&str> {
        self.events.values().map(|e| e.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_from_signature() {
        let sig = EventSignature::parse(
            "Transfer(address indexed from, address indexed to, uint256 value)",
        )
        .unwrap();
        let decoder = LogDecoder::from_signature(&sig).unwrap();

        assert_eq!(decoder.event_names(), vec!["Transfer"]);
    }

    #[test]
    fn test_decoded_value_json() {
        let value = DecodedValue::Address("0x1234567890123456789012345678901234567890".to_string());
        let json = serde_json::to_string(&value).unwrap();
        assert!(json.contains("0x1234567890123456789012345678901234567890"));

        let value = DecodedValue::Uint("1000000".to_string());
        let json = serde_json::to_string(&value).unwrap();
        assert!(json.contains("1000000"));
    }
}

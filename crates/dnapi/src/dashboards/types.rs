//! Types for the Dashboards API

use serde::{Deserialize, Serialize};

/// Where a widget sits on the dashboard grid
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
#[non_exhaustive]
pub struct WidgetPosition {
    /// Column of the top-left corner
    pub col: i64,
    /// Row of the top-left corner
    pub row: i64,
    /// Width in grid columns
    pub size_x: i64,
    /// Height in grid rows
    pub size_y: i64,
}

impl WidgetPosition {
    /// A position at (`col`, `row`) spanning `size_x` by `size_y` cells
    #[must_use]
    pub fn new(col: i64, row: i64, size_x: i64, size_y: i64) -> Self {
        Self {
            col,
            row,
            size_x,
            size_y,
        }
    }
}

/// A query parameter widget on a dashboard
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ParamWidget {
    /// Widget ID
    pub widget_id: Option<String>,
    /// Parameter key
    pub key: Option<String>,
    /// Query the parameter belongs to
    pub query_id: Option<i64>,
    /// Visualization widget the parameter is attached to
    pub visualization_widget_id: Option<i64>,
    /// Grid position
    pub position: Option<WidgetPosition>,
}

/// A text (markdown) widget on a dashboard
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TextWidget {
    /// Widget ID
    pub widget_id: Option<i64>,
    /// Markdown text
    pub text: Option<String>,
    /// Grid position
    pub position: Option<WidgetPosition>,
}

/// A visualization widget on a dashboard
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VisualizationWidget {
    /// Widget ID
    pub widget_id: Option<i64>,
    /// Visualization shown by the widget
    pub visualization_id: Option<i64>,
    /// Grid position
    pub position: Option<WidgetPosition>,
}

/// A dashboard with its widgets
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Dashboard {
    /// Dashboard ID
    pub dashboard_id: i64,
    /// Dashboard name
    pub name: Option<String>,
    /// URL slug
    pub slug: Option<String>,
    /// Dashboard URL on dune.com
    pub dashboard_url: Option<String>,
    /// Whether the dashboard is private
    pub is_private: Option<bool>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Visualization widgets
    #[serde(default)]
    pub visualization_widgets: Vec<VisualizationWidget>,
    /// Text widgets
    #[serde(default)]
    pub text_widgets: Vec<TextWidget>,
    /// Parameter widgets
    #[serde(default)]
    pub param_widgets: Vec<ParamWidget>,
}

/// A text widget to place on a dashboard
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct TextWidgetInput {
    /// Markdown text
    pub text: String,
    /// Grid position; omitted lets Dune place it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<WidgetPosition>,
}

impl TextWidgetInput {
    /// A text widget Dune places itself
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            position: None,
        }
    }

    /// Place the widget at `position`
    #[must_use]
    pub fn at(mut self, position: WidgetPosition) -> Self {
        self.position = Some(position);
        self
    }
}

/// A visualization widget to place on a dashboard
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct VisualizationWidgetInput {
    /// Visualization to show
    pub visualization_id: i64,
    /// Grid position; omitted lets Dune place it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<WidgetPosition>,
}

impl VisualizationWidgetInput {
    /// A visualization widget Dune places itself
    #[must_use]
    pub fn new(visualization_id: i64) -> Self {
        Self {
            visualization_id,
            position: None,
        }
    }

    /// Place the widget at `position`
    #[must_use]
    pub fn at(mut self, position: WidgetPosition) -> Self {
        self.position = Some(position);
        self
    }
}

/// A query parameter widget to place on a dashboard
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct ParamWidgetInput {
    /// Parameter key
    pub key: String,
    /// Query the parameter belongs to
    pub query_id: i64,
    /// Visualization widget to attach the parameter to; omitted makes it a
    /// dashboard-wide parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visualization_widget_id: Option<i64>,
    /// Grid position; omitted lets Dune place it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<WidgetPosition>,
}

impl ParamWidgetInput {
    /// A widget for parameter `key` of query `query_id`
    #[must_use]
    pub fn new(key: impl Into<String>, query_id: i64) -> Self {
        Self {
            key: key.into(),
            query_id,
            visualization_widget_id: None,
            position: None,
        }
    }
}

/// Request to create a dashboard (`POST /v1/dashboards`)
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct CreateDashboardRequest {
    /// Dashboard name
    pub name: String,
    /// Whether the dashboard is private
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
    /// Number of widgets per row when Dune lays them out
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns_per_row: Option<i64>,
    /// Visualizations to add as widgets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visualization_ids: Option<Vec<i64>>,
    /// Text widgets to add
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_widgets: Option<Vec<TextWidgetInput>>,
}

impl CreateDashboardRequest {
    /// An empty dashboard named `name`
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_private: None,
            columns_per_row: None,
            visualization_ids: None,
            text_widgets: None,
        }
    }

    /// Set whether the dashboard is private
    #[must_use]
    pub fn private(mut self, is_private: bool) -> Self {
        self.is_private = Some(is_private);
        self
    }

    /// Add these visualizations as widgets
    #[must_use]
    pub fn visualizations(mut self, ids: Vec<i64>) -> Self {
        self.visualization_ids = Some(ids);
        self
    }

    /// Add these text widgets
    #[must_use]
    pub fn text_widgets(mut self, widgets: Vec<TextWidgetInput>) -> Self {
        self.text_widgets = Some(widgets);
        self
    }
}

/// Request to update a dashboard (`PATCH /v1/dashboards/{dashboard_id}`)
///
/// Fields left `None` are not sent and keep their value. A widget list that
/// is set **replaces** every widget of that kind on the dashboard; an empty
/// list removes them all.
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct UpdateDashboardRequest {
    /// New name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New URL slug
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Whether the dashboard is private
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
    /// Tags (replaces the existing tags)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Number of widgets per row when Dune lays them out
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns_per_row: Option<i64>,
    /// Replacement visualization widgets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visualization_widgets: Option<Vec<VisualizationWidgetInput>>,
    /// Replacement text widgets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_widgets: Option<Vec<TextWidgetInput>>,
    /// Replacement parameter widgets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param_widgets: Option<Vec<ParamWidgetInput>>,
}

impl UpdateDashboardRequest {
    /// A request that changes nothing until fields are set
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Rename the dashboard
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set whether the dashboard is private
    #[must_use]
    pub fn private(mut self, is_private: bool) -> Self {
        self.is_private = Some(is_private);
        self
    }

    /// Replace the tags
    #[must_use]
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }
}

/// Response from archiving a dashboard
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ArchiveDashboardResponse {
    /// Archived dashboard ID
    pub dashboard_id: Option<i64>,
    /// Whether the dashboard was archived
    pub ok: Option<bool>,
}

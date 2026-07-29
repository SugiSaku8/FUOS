use serde::{Deserialize, Serialize};

/// Dataset category inside SOF.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatasetType {
    /// Boundary condition data.
    Boundary,

    /// Restart data.
    Restart,

    /// General ocean field.
    Ocean,
}

impl Default for DatasetType {
    fn default() -> Self {
        Self::Ocean
    }
}

/// Dataset metadata stored in SOF.
///
/// This describes the logical structure of the data.
/// Actual binary storage is handled by s-io.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SofDataset {
    /// Dataset name.
    pub name: String,

    /// SOF version.
    pub version: String,

    /// Dataset type.
    pub dataset_type: DatasetType,

    /// Stored variables.
    pub variables: Vec<VariableInfo>,
}

impl SofDataset {
    pub fn new(name: impl Into<String>, dataset_type: DatasetType) -> Self {
        Self {
            name: name.into(),

            version: "0.1.0".to_string(),

            dataset_type,

            variables: Vec::new(),
        }
    }

    /// Creates standard boundary dataset.
    pub fn boundary() -> Self {
        let mut dataset = Self::new("SOF Boundary Dataset", DatasetType::Boundary);

        dataset.add_variable(VariableInfo::new("velocity_u", VariableType::Velocity));

        dataset.add_variable(VariableInfo::new("velocity_v", VariableType::Velocity));

        dataset.add_variable(VariableInfo::new("velocity_w", VariableType::Velocity));

        dataset.add_variable(VariableInfo::new("temperature", VariableType::Temperature));

        dataset.add_variable(VariableInfo::new("salinity", VariableType::Salinity));

        dataset
    }

    pub fn add_variable(&mut self, variable: VariableInfo) {
        self.variables.push(variable);
    }
}

/// Variable information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    /// Variable name.
    pub name: String,

    /// Variable category.
    pub variable_type: VariableType,
}

impl VariableInfo {
    pub fn new(name: impl Into<String>, variable_type: VariableType) -> Self {
        Self {
            name: name.into(),

            variable_type,
        }
    }
}

/// Physical variable type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VariableType {
    Velocity,

    Temperature,

    Salinity,

    Coordinate,

    Scalar,
}

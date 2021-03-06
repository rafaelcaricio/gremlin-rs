use crate::structure::VertexProperty;
use crate::structure::GID;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Vertex {
    id: GID,
    label: String,
    properties: HashMap<String, Vec<VertexProperty>>,
}

impl Vertex {
    pub fn with_label<T>(id: GID, label: T) -> Vertex
    where
        T: Into<String>,
    {
        Self::new(id, label, HashMap::new())
    }
    pub fn new<T>(id: GID, label: T, properties: HashMap<String, Vec<VertexProperty>>) -> Vertex
    where
        T: Into<String>,
    {
        Vertex {
            id,
            label: label.into(),
            properties,
        }
    }

    pub fn id(&self) -> &GID {
        &self.id
    }

    pub fn label(&self) -> &String {
        &self.label
    }

    pub fn property(&self, key: &str) -> Option<&VertexProperty> {
        self.properties.get(key).and_then(|v| v.get(0))
    }
}

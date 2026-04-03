use serde::Serialize;


#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    
}

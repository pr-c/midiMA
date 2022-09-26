use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ItemGroup {
    pub itemsType: i32,
    pub cntPages: i32,
    pub items: Vec<Vec<Executor>>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Executor {
    pub i: UnknownType1,
    pub oType: UnknownType2,
    pub oI: UnknownType2,
    pub tt: UnknownType2,
    #[serde(rename = "bC")]
    pub text_color: String,
    #[serde(rename = "bdC")]
    pub background_color: String,
    pub cues: Cues,
    #[serde(rename = "combinedItems")]
    pub combined_executor_blocks: i32,
    pub iExec: i32,
    pub isRun: i32,
    #[serde(rename = "executorBlocks")]
    pub executor_blocks: Vec<ExecutorBlock>,
}

#[derive(Serialize, Deserialize)]
pub struct UnknownType1 {
    pub t: String,
    #[serde(rename = "c")]
    pub color: String,
}

#[derive(Serialize, Deserialize)]
pub struct UnknownType2 {
    pub t: String,
}

#[derive(Serialize, Deserialize)]
pub struct Cues {}

#[derive(Serialize, Deserialize)]
pub struct ExecutorBlock {
    pub button1: Button,
    pub button2: Button,
    pub button3: Button,
    pub fader: Fader,
}

#[derive(Serialize, Deserialize)]
pub struct Fader {
    #[serde(rename = "bdC")]
    pub background_color: Option<String>,
    pub tt: Option<String>,
    #[serde(rename = "v")]
    pub value: f32,
    #[serde(rename = "vT")]
    pub value_string: Option<String>,
    pub min: f64,
    pub max: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Button {
    pub id: i32,
    #[serde(rename = "t")]
    pub type_string: String,
    #[serde(rename = "s")]
    pub pressed: bool,
    #[serde(rename = "c")]
    pub text_color: String,
    #[serde(rename = "bdC")]
    pub background_color: Option<String>,
}

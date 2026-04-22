mod context;
mod export_import;
mod extensions;
mod features;
mod files;
mod folders;
mod git;
mod groups;
mod ide;
mod links;
mod mcp;
mod notes;
mod notifications;
mod plans;
mod repos;
mod search;
mod sessions;
mod skills;
mod settings;
mod storage;
mod tags;
mod tasks;
mod terminal;
mod timeline;
mod knowledge;

pub use context::*;
pub use export_import::*;
pub use extensions::*;
pub use features::*;
pub use files::*;
pub use folders::*;
pub use git::*;
pub use groups::*;
pub use ide::*;
pub use links::*;
pub use mcp::*;
pub use notes::*;
pub use notifications::*;
pub use plans::*;
pub use repos::*;
pub use search::*;
pub use sessions::*;
pub use skills::*;
pub use settings::*;
pub use storage::*;
pub use tags::*;
pub use tasks::*;
pub use terminal::*;
pub use timeline::*;
pub use knowledge::*;

#[derive(serde::Serialize)]
pub struct FeatureData {
    pub feature: crate::db::features::Feature,
    pub all_tags: Vec<crate::db::tags::Tag>,
    pub tasks: Vec<crate::db::tasks::Task>,
    pub plans: Vec<crate::db::plans::Plan>,
    pub note: Option<crate::db::notes::Note>,
}

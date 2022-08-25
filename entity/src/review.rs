//! SeaORM Entity. Generated by sea-orm-codegen 0.8.0

use chrono::Local;
use sea_orm::entity::prelude::*;
use sea_orm::{NotSet};
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::course;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "review")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Custom(\"LONGTEXT\".to_owned())")]
    pub title: String,
    #[sea_orm(column_type = "Custom(\"LONGTEXT\".to_owned())")]
    pub content: String,
    pub history: Json,
    pub reviewer_id: i32,
    pub time_created: DateTime,
    pub time_updated: DateTime,
    pub rank: Json,
    pub upvoters: Json,
    pub downvoters: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::course::Entity")]
    Course,
}

impl Related<super::course::Entity> for Entity {
    fn to() -> RelationDef {
        super::course_review::Relation::Course.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::course_review::Relation::Review.def().rev())
    }
}

fn _calculate_votes(model: &Model, user_id: i32) -> (i32, i32, i32) {
    let mut upvote: i32 = 0;
    let mut downvote: i32 = 0;
    let mut voted: i32 = 0;
    if let Some(upvoters) = model.upvoters.as_array() {
        upvote = upvoters.len() as i32;
        if upvoters.contains(&json!(user_id)) {
            voted = 1;
        }
    }
    if let Some(downvoters) = model.downvoters.as_array() {
        downvote = downvoters.len() as i32;
        if downvoters.contains(&json!(user_id)) {
            voted = -1;
        }
    }
    (upvote, downvote, voted)
}

impl GetReview {
    pub fn new(model: Model, user_id: i32) -> Self {
        let (upvote, downvote, voted) = _calculate_votes(&model, user_id);
        GetReview {
            id: model.id,
            title: model.title,
            content: model.content,
            history: model.history,
            reviewer_id: model.reviewer_id,
            time_created: model.time_created,
            time_updated: model.time_updated,
            rank: model.rank,
            is_me: model.reviewer_id == user_id,
            remark: upvote - downvote,
            vote: voted,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetReview {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub history: Json,
    pub reviewer_id: i32,
    pub time_created: DateTime,
    pub time_updated: DateTime,
    pub rank: Json,
    pub is_me: bool,
    pub vote: i32,
    pub remark: i32,
}

impl NewReview {
    pub fn into_active_model(self, user_id: i32) -> ActiveModel {
        let now = Local::now();
        ActiveModel {
            id: NotSet,
            title: Set(self.title),
            content: Set(self.content),
            history: Set(json!([])),
            reviewer_id: Set(user_id),
            time_created: Set(now.naive_utc()),
            time_updated: Set(now.naive_utc()),
            rank: Set(self.rank),
            upvoters: Set(json!([])),
            downvoters: Set(json!([])),
        }
    }
}

impl ActiveModel {
    pub fn update_with(&mut self, updated_review: NewReview) {
        self.title = Set(updated_review.title);
        self.content = Set(updated_review.content);
        self.rank = Set(updated_review.rank);
        self.time_updated = Set(Local::now().naive_utc());
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewReview {
    pub title: String,
    pub content: String,
    pub rank: Json,
}

impl From<Model> for HistoryReview {
    fn from(model: Model) -> Self {
        HistoryReview {
            title: model.title,
            content: model.content,
            reviewer_id: model.reviewer_id,
            time_created: model.time_created,
            time_updated: model.time_updated,
            rank: model.rank,
        }
    }
}

impl GetMyReview {
    pub fn new(model: Model, course: course::Model, user_id: i32) -> Self {
        let (upvote, downvote, voted) = _calculate_votes(&model, user_id);
        GetMyReview {
            id: model.id,
            title: model.title,
            content: model.content,
            history: model.history,
            time_created: model.time_created,
            time_updated: model.time_updated,
            rank: model.rank,
            remark: upvote - downvote,
            vote: voted,
            course,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetMyReview {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub history: Json,
    pub time_created: DateTime,
    pub time_updated: DateTime,
    pub rank: Json,
    pub vote: i32,
    pub remark: i32,
    pub course: course::Model,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoryReview {
    pub title: String,
    pub content: String,
    pub reviewer_id: i32,
    pub time_created: DateTime,
    pub time_updated: DateTime,
    pub rank: Json,
}


impl ActiveModelBehavior for ActiveModel {}
//! Category models
//!
//! Hierarchical product categories with self-referencing parent_id.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// A product category, supporting tree hierarchy via `parent_id`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A category node with its children for tree rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTree {
    #[serde(flatten)]
    pub category: Category,
    pub children: Vec<CategoryTree>,
}

/// Request payload for creating a category.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCategoryRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

/// Request payload for updating a category.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCategoryRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

impl CategoryTree {
    /// Build a category tree from a flat list of categories.
    pub fn build_tree(categories: Vec<Category>) -> Vec<CategoryTree> {
        let root_categories: Vec<&Category> =
            categories.iter().filter(|c| c.parent_id.is_none()).collect();

        root_categories
            .into_iter()
            .map(|root| Self::build_subtree(root.clone(), &categories))
            .collect()
    }

    fn build_subtree(parent: Category, all: &[Category]) -> CategoryTree {
        let children: Vec<CategoryTree> = all
            .iter()
            .filter(|c| c.parent_id == Some(parent.id))
            .map(|child| Self::build_subtree(child.clone(), all))
            .collect();

        CategoryTree {
            category: parent,
            children,
        }
    }
}

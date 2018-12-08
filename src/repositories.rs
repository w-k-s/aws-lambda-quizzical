use log::{error, info};
use models;
use models::{Categories, Category, Choice, Question};
use postgres::Connection;
use std::collections::HashMap;
use std::fmt;

pub enum RepositoryError {
    DatabaseError(String),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let RepositoryError::DatabaseError(ref msg) = *self;
        write!(f, "{}", msg)
    }
}

pub struct CategoriesRepository {
    //moving the connection into repo because each lambda invokation will trigger a new instance.
    //TODO: setup a singleton connection pool
    pub conn: Connection,
}

impl CategoriesRepository {
    pub fn list_categories(&self) -> Result<Categories, RepositoryError> {
        let rows = &self
            .conn
            .query("SELECT name FROM categories", &[])
            .map_err(|e| RepositoryError::DatabaseError(format!("{}", e)))?;

        let mut categories: Vec<Category> = Vec::with_capacity(rows.len());

        for row in rows {
            categories.push(Category { title: row.get(0) });
        }

        Ok(Categories {
            categories: categories,
        })
    }
}

pub struct QuestionsRepository {
    //moving the connection into repo because each lambda invokation will trigger a new instance.
    //TODO: setup a singleton connection pool
    pub conn: Connection,
}

type TotalRecordsCount = i64;

impl QuestionsRepository {
    pub fn get_questions(
        &self,
        category: &str,
        page: i64,
        size: i64,
    ) -> Result<(Vec<Question>, TotalRecordsCount), RepositoryError> {
        let offset = match page {
            0 => 0i64,
            _ => (page - 1i64) * size,
        };

        let count_rows = &self
            .conn
            .query(
                "SELECT COUNT(id) FROM questions WHERE category = $1",
                &[&category],
            )
            .map_err(|e| {
                error!(
                    "Error counting questions for category '{}': {}",
                    category, e
                );
                RepositoryError::DatabaseError(format!("{}", e))
            })?;

        if count_rows.is_empty() {
            return Ok((vec![], 0));
        }

        let count: i64 = count_rows.get(0).get(0);

        let question_rows = &self
            .conn
            .query(
                "SELECT id,text FROM questions WHERE category = $1 LIMIT $2 OFFSET $3",
                &[&category, &size, &offset],
            )
            .map_err(|e| {
                error!("Error loading questions for category '{}': {}", category, e);
                RepositoryError::DatabaseError(format!("{}", e))
            })?;

        let mut question_ids: Vec<i64> = vec![];
        for question_row in question_rows {
            let id: i64 = question_row.get(0);
            question_ids.push(id);
        }

        info!("Loading choices for questions '{:?}'", question_ids);

        let choices_rows = &self
            .conn
            .query(
                "SELECT text,correct,question_id FROM choices WHERE question_id = ANY($1)",
                &[&question_ids],
            )
            .map_err(|e| {
                error!(
                    "Error loading choices for questions '{:?}': {}",
                    question_ids, e
                );
                RepositoryError::DatabaseError(format!("{}", e))
            })?;

        let mut choices_map: HashMap<i64, Vec<Choice>> = HashMap::new();
        for choice_row in choices_rows {
            let question_id: i64 = choice_row.get(2);
            let choice = Choice {
                title: choice_row.get(0),
                correct: choice_row.get(1),
            };

            if let Some(mut choices) = choices_map.get_mut(&question_id) {
                choices.push(choice);
                continue;
            }

            choices_map.insert(question_id, vec![choice]);
        }

        let mut questions: Vec<Question> = Vec::with_capacity(question_rows.len());
        for question_row in question_rows {
            let id: i64 = question_row.get(0);
            let text: String = question_row.get(1);
            let choices: Vec<Choice> = choices_map.get(&id).unwrap_or(&vec![]).to_vec();

            questions.push(Question {
                id: id,
                question: text,
                category: category.to_string(),
                choices: choices,
            });
        }

        Ok((questions, count))
    }
}

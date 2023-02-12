use crate::{
    domain::model::{Article, UserId},
    repos::{ArticlesRepo, UsersRepo},
    AppError,
};
use serde::Deserialize;
use slug::slugify;
use std::sync::Arc;

#[derive(Clone)]
pub struct ArticlesMgr {
    articles_repo: ArticlesRepo,
    user_repo: Arc<UsersRepo>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticleInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    #[serde(rename = "tagList")]
    pub tag_list: Option<Vec<String>>,
}

impl ArticlesMgr {
    /// Create a new instance of `ArticlesMgr`.
    pub fn new(articles_repo: ArticlesRepo, user_repo: Arc<UsersRepo>) -> Self {
        Self {
            articles_repo,
            user_repo,
        }
    }

    pub async fn get_articles(&self) -> Result<Vec<Article>, AppError> {
        self.articles_repo.get_articles().await
    }

    pub async fn get_article(&self, slug: String) -> Result<Option<Article>, AppError> {
        self.articles_repo.get_article(slug).await
    }

    pub async fn create_article(
        &self,
        title: String,
        description: String,
        body: String,
        tag_list: Vec<String>,
        author_id: UserId,
    ) -> Result<Article, AppError> {
        //
        let slug = slugify(&title);
        let mut a = Article::new_basic(
            slug,
            title,
            description,
            body,
            tag_list,
            author_id.as_value(),
        );
        self.articles_repo.add(&mut a).await?;
        a.author = self
            .user_repo
            .get_profile_by_id(author_id.as_value())
            .await?;
        Ok(a)
    }

    pub async fn delete_article(&self, slug: String) -> Result<(), AppError> {
        //
        self.articles_repo.delete(slug).await
    }

    pub async fn update_article(
        &self,
        slug: String,
        input: UpdateArticleInput,
    ) -> Result<Article, AppError> {
        //
        log::debug!("update_article >> input={:?}", input);
        let res = self.get_article(slug).await?;
        if res.is_none() {
            return Err(AppError::NotFound("article".into()));
        }
        let mut a = res.unwrap();

        // Fill-in any of the input's elements.
        if let Some(title) = input.title {
            a.title = title;
        }
        if let Some(description) = input.description {
            a.description = description;
        }
        if let Some(body) = input.body {
            a.body = body;
        }
        if let Some(tag_list) = input.tag_list {
            a.tag_list = tag_list;
        }

        self.articles_repo.update(&mut a).await.map(|_| a)
    }
}

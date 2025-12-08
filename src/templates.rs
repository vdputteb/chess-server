// src/templates.rs
use askama::Template;
use axum::response::Html;

// Template pour la page d'accueil
#[derive(Template)]
#[template(path = "account.html")]
pub struct AccountTemplate;

// Template pour la page de login
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate;

// Template pour la page solo
#[derive(Template)]
#[template(path = "solo.html")]
pub struct SoloTemplate;

// Template pour la page online
#[derive(Template)]
#[template(path = "online.html")]
pub struct OnlineTemplate;

// Template pour la page paramètres
#[derive(Template)]
#[template(path = "param.html")]
pub struct ParamTemplate;

// Handlers pour servir les pages
pub async fn serve_account() ->Html<String>{
    Html(AccountTemplate.render().unwrap())
}

pub async fn serve_login() -> Html<String> {
    Html(LoginTemplate.render().unwrap())
}

pub async fn serve_solo() -> Html<String> {
    Html(SoloTemplate.render().unwrap())
}

pub async fn serve_online() -> Html<String> {
    Html(OnlineTemplate.render().unwrap())
}

pub async fn serve_param() -> Html<String> {
    Html(ParamTemplate.render().unwrap())
}
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection};

mod errors;

#[derive(Deserialize, Debug)]
struct Auth {
    token: Option<String>,
}

#[derive(Serialize, Debug)]
struct User {
    name: String,
}

fn auth_filter() -> impl Filter<Extract = (User,), Error = Rejection> + Clone {
    let auth_header = warp::header("authorization").and_then(|header: String| async move {
        let mut split = header.splitn(2, ' ');
        match (split.next(), split.next()) {
            (Some("Bearer"), Some(token)) => Ok(Auth {
                token: Some(token.to_string()),
            }),
            (Some(_), Some(_)) => Err(errors::auth_token("invalid scheme")),
            (_, _) => Err(errors::auth_token("missing token")),
        }
    });

    let auth_query = warp::query::<Auth>();

    auth_header
        .or(auth_query)
        .unify()
        .and_then(|auth: Auth| async move {
            match auth.token {
                Some(_token) => Ok(User {
                    name: "belak".to_string(),
                }),
                None => Err(errors::unauthorized()),
            }
        })
        .recover(errors::handle_rejection)
}

fn display_user(user: User) -> impl warp::Reply {
    warp::reply::json(&user)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let auth = auth_filter();

    let example = warp::get()
        .and(warp::path("example"))
        .and(auth)
        .map(display_user)
        .recover(errors::handle_rejection);

    warp::serve(example).run(([127, 0, 0, 1], 3030)).await
}

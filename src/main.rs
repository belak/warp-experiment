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
    warp::header::optional("authorization")
        .and(warp::query::<Auth>())
        .and_then(|header: Option<String>, query: Auth| async move {
            let token = match (header.as_ref(), &query.token) {
                (Some(header), None) => {
                    let mut split = header.splitn(2, ' ');
                    match (split.next(), split.next()) {
                        (Some("Bearer"), Some(token)) => token,
                        (Some(_), Some(_)) => return Err(errors::auth_token("invalid scheme")),
                        (_, _) => return Err(errors::auth_token("missing token")),
                    }
                }
                (None, Some(token)) => token.as_str(),
                (Some(_), Some(_)) => return Err(errors::auth_token("multiple tokens specified")),
                (None, None) => return Err(errors::auth_token("no token specified")),
            };

            if token != "hello world" {
                return Err(errors::unauthorized());
            }

            Ok(User {
                name: "belak".to_string(),
            })
        })
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

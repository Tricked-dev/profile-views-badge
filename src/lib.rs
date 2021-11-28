use badge_maker::BadgeBuilder;
use std::collections::HashMap;
use worker::*;
mod consts;

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    let router = Router::new();
    router
        .get_async("/badge", |req, ctx| async move {
            let pairs: HashMap<_, _> = req.url().unwrap().query_pairs().into_owned().collect();
            let default_user = &"user_not_found".to_owned();

            let username = pairs.get("user").unwrap_or(default_user);

            let profile_views = ctx.kv("PROFILE_VIEWS").unwrap();
            let views: String = match profile_views.get(username).await.unwrap() {
                Some(views_value) => {
                    let views: i32 = views_value.as_string().parse().unwrap();
                    profile_views
                        .put(username, views + 1)
                        .unwrap()
                        .execute()
                        .await
                        .unwrap();
                    format!("{}", views)
                }
                None => {
                    profile_views
                        .put(username, 2)
                        .unwrap()
                        .execute()
                        .await
                        .unwrap();
                    "1".to_owned()
                }
            };
            console_log!("views:{}", views);
            let mut badge = BadgeBuilder::new();

            badge
                .label(pairs.get("label").unwrap_or(&"Profile views".to_owned()))
                .message(&views);

            if let Some(logo) = pairs.get("logo") {
                badge.logo_url(logo);
            }
            if let Some(link) = pairs.get("link") {
                badge.link(link);
            }
            if let Some(style) = pairs.get("style") {
                badge.style_parse(style);
            }

            if let Some(color) = pairs.get("color") {
                badge.color_parse(color);
            } else {
                badge.color_parse("#007ec6");
            }

            if let Some(logo_url) = pairs.get("logo_url") {
                badge.logo_url(logo_url);
            }
            if let Some(logo_width) = pairs.get("logo_width") {
                let width = logo_width.trim().parse();
                if let Ok(width) = width {
                    badge.logo_width(width);
                }
            }
            if let Some(logo_padding) = pairs.get("logo_padding") {
                let padding = logo_padding.trim().parse();
                if let Ok(padding) = padding {
                    badge.logo_padding(padding);
                }
            }

            let mut headers = Headers::new();
            headers.append("content-type", "image/svg+xml").unwrap();
            let badge_data = badge.build();
            if let Ok(data_badge) = badge_data {
                let svg = data_badge.svg();
                console_log!("SVG: {}", svg);
                Ok(Response::from_bytes(svg.as_bytes().to_vec())
                    .unwrap()
                    .with_headers(headers))
            } else {
                Ok(Response::from_bytes(consts::SVG_ERROR.to_vec())
                    .unwrap()
                    .with_headers(headers))
            }
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}

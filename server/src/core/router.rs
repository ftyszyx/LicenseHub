use crate::apis::{self, *};
use crate::core::app::AppState;
use salvo::cors::{AllowHeaders, AllowOrigin, Cors};
use salvo::http::Method;
use salvo::prelude::*;
use salvo_oapi::security::{Http, HttpAuthScheme};
use salvo_oapi::{OpenApi, SecurityScheme};

pub fn create_router(app_state: AppState) -> Service {
    let admin_routes = Router::with_path("/api/admin")
        .hoop(auth_middleware::auth)
        .hoop(auth_middleware::error_handler)
        .push(Router::with_path("me").get(apis::auth::get_current_user))
        //users
        .push(Router::with_path("me/password").post(apis::auth::change_password))
        .push(Router::with_path("users").post(apis::user_handler::add))
        .push(Router::with_path("users/list").get(apis::user_handler::get_list))
        .push(Router::with_path("users/{id}").get(apis::user_handler::get_by_id))
        .push(Router::with_path("users/{id}").put(apis::user_handler::update))
        .push(Router::with_path("users/{id}").delete(apis::user_handler::delete))
        //apps
        .push(Router::with_path("apps").post(apis::app_handler::add))
        .push(Router::with_path("apps/list").get(apis::app_handler::get_list))
        .push(Router::with_path("apps/{id}").get(apis::app_handler::get_by_id))
        .push(Router::with_path("apps/{id}").put(apis::app_handler::update))
        .push(Router::with_path("apps/{id}").delete(apis::app_handler::delete))
        //roles
        .push(Router::with_path("roles").post(apis::role_handler::add))
        .push(Router::with_path("roles/list").get(apis::role_handler::get_list))
        .push(Router::with_path("roles/{id}").get(apis::role_handler::get_by_id))
        .push(Router::with_path("roles/{id}").put(apis::role_handler::update))
        .push(Router::with_path("roles/{id}").delete(apis::role_handler::delete))
        //products
        //reg_codes
        .push(Router::with_path("reg_codes").post(apis::reg_codes_handler::add))
        .push(Router::with_path("reg_codes/list").get(apis::reg_codes_handler::get_list))
        .push(Router::with_path("reg_codes/{id}").get(apis::reg_codes_handler::get_by_id))
        .push(Router::with_path("reg_codes/{id}").put(apis::reg_codes_handler::update))
        .push(Router::with_path("reg_codes/{id}").delete(apis::reg_codes_handler::delete))
        //devices
        .push(Router::with_path("devices/list").get(apis::device_handler::get_list));

    let cors = Cors::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        // .allow_headers(vec!["authorization","content-type"]).into_handler();
        .allow_headers(AllowHeaders::any())
        .into_handler();
    let register_open = app_state.config.register_open;
    let mut router = Router::new()
        .hoop(affix_state::inject(app_state))
        .push(Router::with_path("/api/login").post(apis::auth::login))
        .push(Router::with_path("/api/reg/validate").post(apis::reg_codes_handler::validate_code))
        .push(Router::with_path("/api/reg/validate").post(apis::reg_codes_handler::validate_code))
        .push(
            Router::with_path("/api/reg/validate").get(apis::reg_codes_handler::validate_code_get),
        )
        .push(admin_routes);
    if register_open {
        router = router.push(Router::with_path("/api/register").post(apis::auth::register));
    }
    //添加swagger-ui
    let doc = OpenApi::new("app_server_api", "1.0.0")
        .add_security_scheme(
            "bearer",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer).bearer_format("JWT")),
        )
        .merge_router(&router);
    let router = router
        .unshift(doc.into_router("/api-doc/openapi.json"))
        .unshift(SwaggerUi::new("/api-doc/openapi.json").into_router("/swagger-ui"));
    let service = Service::new(router).hoop(cors).hoop(Logger::new());
    service
}

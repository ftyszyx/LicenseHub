use crate::apis::{self, *};
use crate::core::app::AppState;
use crate::core::rbac::RequirePerm;
use crate::apis::log_middleware;
use salvo::cors::{AllowHeaders, AllowOrigin, Cors};
use salvo::http::Method;
use salvo::prelude::*;
use salvo_oapi::security::{Http, HttpAuthScheme};
use salvo_oapi::{OpenApi, SecurityScheme};

#[handler]
async fn health_check(res: &mut Response) {
    res.render("hello world");
}

pub fn create_router(app_state: AppState) -> Service {
    let admin_routes = Router::with_path("/api/admin")
        .hoop(auth_middleware::auth)
        .hoop(auth_middleware::error_handler)
        .push(Router::with_path("me").get(apis::auth::get_current_user))
        .push(Router::with_path("me/permissions").get(apis::auth::get_my_permissions))
        //permissions
        .push(
            Router::with_path("permissions/list")
                .hoop(RequirePerm::new("roles", "READ"))
                .get(apis::permission_handler::list_permissions),
        )
        .push(
            Router::with_path("roles/{role_id}/permissions")
                .hoop(RequirePerm::new("roles", "READ"))
                .get(apis::permission_handler::get_role_permissions),
        )
        .push(
            Router::with_path("roles/{role_id}/permissions")
                .hoop(RequirePerm::new("roles", "UPDATE"))
                .post(apis::permission_handler::set_role_permissions),
        )
        //users
        .push(
            Router::with_path("me/password")
                .hoop(RequirePerm::new("me", "UPDATE"))
                .post(apis::auth::change_password),
        )
        .push(
            Router::with_path("users")
                .hoop(RequirePerm::new("users", "CREATE"))
                .post(apis::user_handler::add),
        )
        .push(
            Router::with_path("users/list")
                .hoop(RequirePerm::new("users", "READ"))
                .get(apis::user_handler::get_list),
        )
        .push(
            Router::with_path("users/{id}")
                .hoop(RequirePerm::new("users", "READ"))
                .get(apis::user_handler::get_by_id),
        )
        .push(
            Router::with_path("users/{id}")
                .hoop(RequirePerm::new("users", "UPDATE"))
                .put(apis::user_handler::update),
        )
        .push(
            Router::with_path("users/{id}")
                .hoop(RequirePerm::new("users", "DELETE"))
                .delete(apis::user_handler::delete),
        )
        //apps
        .push(
            Router::with_path("apps")
                .hoop(RequirePerm::new("apps", "CREATE"))
                .post(apis::app_handler::add),
        )
        .push(
            Router::with_path("apps/list")
                .hoop(RequirePerm::new("apps", "READ"))
                .get(apis::app_handler::get_list),
        )
        .push(
            Router::with_path("apps/{id}")
                .hoop(RequirePerm::new("apps", "READ"))
                .get(apis::app_handler::get_by_id),
        )
        .push(
            Router::with_path("apps/{id}")
                .hoop(RequirePerm::new("apps", "UPDATE"))
                .put(apis::app_handler::update),
        )
        .push(
            Router::with_path("apps/{id}") .hoop(RequirePerm::new("apps", "DELETE")) .delete(apis::app_handler::delete),
        )
        //roles
        .push(
            Router::with_path("roles")
                .hoop(RequirePerm::new("roles", "CREATE"))
                .post(apis::role_handler::add),
        )
        .push(
            Router::with_path("roles/list")
                .hoop(RequirePerm::new("roles", "READ"))
                .get(apis::role_handler::get_list),
        )
        .push(
            Router::with_path("roles/{id}")
                .hoop(RequirePerm::new("roles", "READ"))
                .get(apis::role_handler::get_by_id),
        )
        .push(
            Router::with_path("roles/{id}")
                .hoop(RequirePerm::new("roles", "UPDATE"))
                .put(apis::role_handler::update),
        )
        .push(
            Router::with_path("roles/{id}")
                .hoop(RequirePerm::new("roles", "DELETE"))
                .delete(apis::role_handler::delete),
        )
        //products
        //reg_codes
        .push(
            Router::with_path("reg_codes")
                .hoop(RequirePerm::new("reg_codes", "CREATE"))
                .post(apis::reg_codes_handler::add),
        )
        .push(
            Router::with_path("reg_codes/list")
                .hoop(RequirePerm::new("reg_codes", "READ"))
                .get(apis::reg_codes_handler::get_list),
        )
        .push(
            Router::with_path("reg_codes/{id}")
                .hoop(RequirePerm::new("reg_codes", "READ"))
                .get(apis::reg_codes_handler::get_by_id),
        )
        .push(
            Router::with_path("reg_codes/{id}")
                .hoop(RequirePerm::new("reg_codes", "UPDATE"))
                .put(apis::reg_codes_handler::update),
        )
        .push(
            Router::with_path("reg_codes/{id}/status")
                .hoop(RequirePerm::new("reg_codes", "UPDATE"))
                .put(apis::reg_codes_handler::update_status),
        )
        .push(
            Router::with_path("reg_codes/{id}")
                .hoop(RequirePerm::new("reg_codes", "DELETE"))
                .delete(apis::reg_codes_handler::delete),
        )
        //devices
        .push(
            Router::with_path("devices/list")
                .hoop(RequirePerm::new("devices", "READ"))
                .get(apis::device_handler::get_list),
        );

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
        .get(health_check)
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
    let service = Service::new(router)
        .hoop(cors)
        .hoop(Logger::new())
        .hoop(log_middleware::log_response_body);
    service
}

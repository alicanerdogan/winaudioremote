#![windows_subsystem = "windows"]
use actix_web::{
    body, dev, get, http, middleware::errhandlers, patch, web, App, Error, HttpRequest,
    HttpResponse, HttpServer, Responder,
};
use audiodevice::{export_dll, set_default_audio_device, AudioDevice};
use futures::future::{ready, Ready};
use serde::ser;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

mod audiodevice;
mod registry;

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html")))
}

#[derive(Serialize)]
struct AudioDeviceRef {
    id: String,
    name: String,
    is_default: bool,
}

impl Serialize for AudioDevice {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut state = serializer.serialize_struct("AudioDevice", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("is_default", &self.is_default)?;
        state.end()
    }
}

#[derive(Serialize)]
struct AudioDevicesResponse {
    page: u64,
    page_size: u64,
    items: Vec<AudioDevice>,
}

#[derive(Debug, Deserialize)]
struct AudioDevicePatchRequest {
    is_default: bool,
}

#[derive(Debug, Deserialize)]
struct MasterVolumeRequest {
    level: u64,
}

#[derive(Debug, Serialize)]
struct MasterVolumeResponse {
    level: u64,
}

// Responder
impl Responder for AudioDevicesResponse {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

#[get("/api/audio_devices")]
async fn get_audio_devices() -> impl Responder {
    let devices = audiodevice::get_devices().unwrap();

    AudioDevicesResponse {
        page: 0,
        page_size: 10,
        items: devices,
    }
}

#[patch("/api/audio_devices/{id}")]
async fn patch_audio_device(
    path: web::Path<(String,)>,
    req: web::Json<AudioDevicePatchRequest>,
) -> Result<String, Error> {
    let audio_device_id = path.into_inner().0;
    if req.is_default {
        let devices = audiodevice::get_devices().unwrap();
        devices
            .iter()
            .find(|dev| dev.id == audio_device_id)
            .unwrap();
        set_default_audio_device(&audio_device_id).unwrap();
    }
    Ok(format!("Welcome {} {}!", audio_device_id, req.is_default))
}

// Responder
impl Responder for MasterVolumeResponse {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

#[get("/api/master_volume")]
async fn get_master_volume() -> impl Responder {
    let master_volume = audiodevice::get_master_volume().unwrap();

    MasterVolumeResponse {
        level: master_volume,
    }
}

#[patch("/api/master_volume")]
async fn patch_master_volume(req: web::Json<MasterVolumeRequest>) -> Result<String, Error> {
    audiodevice::set_master_volume(req.level).unwrap();
    Ok(String::from(""))
}

fn handle_bad_request<B>(
    res: dev::ServiceResponse<B>,
) -> Result<errhandlers::ErrorHandlerResponse<body::Body>, Error> {
    let error_msg: String = match res.response().error() {
        Some(e) => format!("{:?}", e),
        None => String::from("Unknown Error"),
    };
    let new_res: dev::ServiceResponse<body::Body> = res.map_body(|_head, _body| {
        body::ResponseBody::Other(body::Body::Message(Box::new(error_msg)))
    });
    Ok(errhandlers::ErrorHandlerResponse::Response(new_res))
}

#[actix_web::main]
async fn main() {
    export_dll();

    registry::add_app_to_startup();

    std::env::set_var("RUST_LOG", "actix_web=debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    let server = HttpServer::new(|| {
        App::new()
            .wrap(
                errhandlers::ErrorHandlers::new()
                    .handler(http::StatusCode::BAD_REQUEST, handle_bad_request),
            )
            .service(index)
            .service(patch_audio_device)
            .service(get_audio_devices)
            .service(get_master_volume)
            .service(patch_master_volume)
    })
    .bind("0.0.0.0:3030")
    .unwrap()
    .run();

    let mut systray_app = systray::Application::new().unwrap();
    let icon = include_bytes!("../static/systray.ico").to_vec();
    systray_app.set_tooltip("winaudioremote").unwrap();
    systray_app
        .set_icon_from_buffer(&icon[0..icon.len()], 256, 256)
        .unwrap();
    systray_app
        .add_menu_item("Quit", move |window| {
            window.quit();
            let _res = server.stop(false);
            Ok::<_, systray::Error>(())
        })
        .unwrap();
    systray_app.wait_for_message().unwrap();
}

#[macro_use]
extern crate rocket;


use std::error::Error;

use rocket::http::Method;
use rocket::response::content::Json;
use rocket_cors::{AllowedOrigins, CorsOptions};
use virt::connect;
use virt::connect::Connect;
use virt::domain::Domain;

use models::FrontResult;
use models::InsModel;

mod models;


#[get("/close/<name>")]
fn close(name: &str) -> Json<String> {
    let conn = get_conn();

    let dom: Domain = match Domain::lookup_by_name(&conn, name) {
        Ok(dom) => dom,
        Err(err) => return rv(&err.message, 400)
    };

    return match dom.shutdown() {
        Ok(_) => rv("已发送关机命令", 200),
        Err(err) => rv(&err.message, err.code)
    };
}

#[get("/open/<name>")]
fn open(name: &str) -> Json<String> {
    let conn = get_conn();

    let dom: Domain = match Domain::lookup_by_name(&conn, name) {
        Ok(dom) => dom,
        Err(err) => return rv(&err.message, 400)
    };

    return match dom.create_with_flags(0) {
        Ok(_) => rv("开机成功", 200),
        Err(err) => rv(&err.message, 200)
    };
}

pub fn rv(msg: &str, code: i32) -> Json<String> {
    let v = FrontResult::new(msg, code);
    println!("{} ", v.haha());
    Json(serde_json::to_string(&v).unwrap())
}


#[get("/")]
fn list() -> Json<String> {
    let flags = connect::VIR_CONNECT_LIST_DOMAINS_ACTIVE | connect::VIR_CONNECT_LIST_DOMAINS_INACTIVE;

    let mut v: Vec<InsModel> = Vec::new();

    let doms = get_conn().list_all_domains(flags).unwrap();

    let mut i = 0;
    for dom in doms {
        i = i + 1;
        let id = dom.get_id().unwrap_or(0);
        let name = dom.get_name().unwrap_or_else(|_| String::from("no-name"));
        let active = dom.is_active().unwrap_or(false);
        let dinfo = dom.get_info().unwrap();
        let tmp = InsModel {
            ins_id: id,
            state: dinfo.state,
            ins_name: name,
            cpu: dinfo.nr_virt_cpu,
            mem: dinfo.memory / 1024 / 1024,
            active,
            key: i,
        };
        v.push(tmp)
    }
    Json(serde_json::to_string(&v).unwrap())
}

fn get_conn() -> Connect {
    let uri = String::from("qemu+ssh://root@192.168.50.20/system?socket=/var/run/libvirt/libvirt-sock");
    return match Connect::open(&uri) {
        Ok(c) => c,
        Err(e) => panic!("No connection to hypervisor: {}", e.message),
    };
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true)
        .to_cors()?;
    rocket::build()
        .mount("/", routes![list,close,open])
        .attach(cors)
        .launch()
        .await?;
    Ok(())
}


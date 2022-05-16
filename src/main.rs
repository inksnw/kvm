#[macro_use]
extern crate rocket;


use std::error::Error;
use std::fs;

use rocket::http::Method;
use rocket::response::content::Json;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::time::{self, Duration};
use rocket_cors::{AllowedOrigins, CorsOptions};
use virt::connect;
use virt::connect::Connect;
use virt::domain::Domain;
use virt::domain::DomainState;

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


#[get("/events")]
fn events() -> EventStream![] {
    EventStream! {
        let mut interval = time::interval(Duration::from_secs(1));
        for i in 0..100{
             yield Event::data(format!("{}", i));
            interval.tick().await;
        }
    }
}

#[get("/clone/<name>")]
fn clone(name: &str) -> Json<String> {
    let conn = get_conn();

    let dom: Domain = match Domain::lookup_by_name(&conn, name) {
        Ok(dom) => dom,
        Err(err) => return rv(&err.message, 400)
    };
    // let xml = dom.get_xml_desc(0).unwrap();
    // println!("{}", xml);

    let contents = fs::read_to_string("/Users/inksnw/Desktop/kvm/src/a.xml")
        .expect("Something went wrong reading the file");
    Domain::define_xml(&conn, &*contents);



    return rv("已发克隆命令", 200);
}


#[get("/open/<name>")]
fn open(name: &str) -> Json<String> {
    let conn = get_conn();

    let dom: Domain = match Domain::lookup_by_name(&conn, name) {
        Ok(dom) => dom,
        Err(err) => return rv(&err.message, 400)
    };

    return match dom.create_with_flags(0) {
        Ok(_) => rv("已发送开机命令", 200),
        Err(err) => rv(&err.message, 200)
    };
}

pub fn rv(msg: &str, code: i32) -> Json<String> {
    let v = FrontResult::new(msg, code);
    println!("{} ", v.haha());
    Json(serde_json::to_string(&v).unwrap())
}

fn get_state(state: DomainState) -> String {
    match state {
        1 => "运行中".to_string(),
        5 => "已关机".to_string(),
        _ => "未定义".to_string()
    }
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
            state: get_state(dinfo.state),
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
        .mount("/", routes![list,close,open,events,clone])
        .attach(cors)
        .launch()
        .await?;
    Ok(())
}


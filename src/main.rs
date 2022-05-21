use std::{fs, process::exit};

const X: u8 = 4;
const Y: u8 = 2;
const Z: u8 = 6;

mod vector_tile;

use protobuf::Message;
use vector_tile::{
    tile::{Feature, GeomType},
    Tile,
};

type Coord = [f64; 2];

fn main() {
    // let feature_collection: Vec<Point> = Vec::new();

    let file_name = format!("src/points_{}-{}-{}.mvt", X, Y, Z);

    let contents = fs::read(file_name).expect("Something went wrong reading the file");
    let tile = Tile::parse_from_bytes(&contents).expect("Something went wrong parsing the file");

    let layer = &tile.layers[0];
    // layer.extent.expect("Layer must provider an extent");
    let extent = 4096_u32;

    let size = extent * 2_u32.pow(Z as u32);
    let x0 = extent * X as u32;
    let y0 = extent * Y as u32;

    let project = |line: &Vec<Coord>| -> Vec<Coord> {
        let mut new_line: Vec<Coord> = Vec::with_capacity(line.len());
        for item in line.iter() {
            let mut p: Coord = [0_f64; 2];

            p[0] = ((item[0] + x0 as f64) * 360_f64) / (size as f64) - 180_f64;

            let y2 = 180_f64 - ((item[1] + y0 as f64) * 360_f64) / (size as f64);
            let aux = std::f64::consts::E.powf((y2 * std::f64::consts::PI) / 180_f64);
            p[1] = (360_f64 / std::f64::consts::PI) * libm::atan(aux) - 90_f64;

            new_line.push(p);
        }

        new_line
    };

    let local_project = |line: &Vec<Coord>| -> Vec<Coord> {
        let mut new_line: Vec<Coord> = Vec::with_capacity(line.len());
        for item in line.iter() {
            let mut p: Coord = [0_f64; 2];

            p[0] = item[0] / extent as f64;
            p[1] = item[1] / extent as f64;

            new_line.push(p);
        }

        new_line
    };

    for item in layer.features.iter() {
        to_geojson(item, project);
        to_geojson(item, local_project);
    }
}

// Aux
fn to_geojson(feature: &Feature, project: impl Fn(&Vec<Coord>) -> Vec<Coord>) {
    let geometry = load_geometry(&feature.geometry);

    let feature_type = feature
        .type_
        .expect("Feature must provide a type")
        .enum_value()
        .unwrap();

    match feature_type {
        GeomType::POINT => {
            let coordinates = project(&geometry[0]);
            println!("{:?}", coordinates);
        }
        GeomType::UNKNOWN => todo!(),
        GeomType::LINESTRING => todo!(),
        GeomType::POLYGON => todo!(),
    }
}

fn load_geometry(geom: &Vec<u32>) -> Vec<Vec<Coord>> {
    let mut idx = 0_usize;

    let mut length = 0_i128;

    let mut cmd = 1_u8;
    let mut x = 0_f64;
    let mut y = 0_f64;

    let mut lines: Vec<Vec<Coord>> = Vec::new();
    let mut line: Vec<Coord> = Vec::new();

    loop {
        if idx >= geom.len() {
            break;
        }

        if length == 0 {
            cmd = (geom[idx] & 0x7) as u8;
            length = (geom[idx] >> 3) as i128;

            idx += 1;
        }

        length -= 1;

        if cmd == 1 || cmd == 2 {
            x += geom[idx] as f64;
            y += geom[idx + 1] as f64;

            idx += 2;

            if cmd == 1 {
                // moveTo
                if line.len() > 0 {
                    lines.push(line.clone());
                }
                line = Vec::new();
            }

            if line.len() == 0 {
                line.push([x, y]);
            }
        } else if cmd == 7 {
            if line.len() > 0 {
                line.push(line[0].clone()); // closePolygon
            }
        } else {
            exit(0);
        }
    }

    if line.len() > 0 {
        lines.push(line);
    }

    lines
}

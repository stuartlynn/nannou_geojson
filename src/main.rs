use geo_types::{Polygon, MultiPolygon};
use geojson::{GeoJson, Geometry, Value};
use nannou::draw::Draw;
use nannou::prelude::*;
use nannou::ui::prelude::*;
use std::convert::TryInto;
use std::fs;


fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    shapes: Vec<Option<Vec<Point3<f32>>>>,
    rotation: f32,
    scale: f32,
    ui: Ui,
    ids: Ids
}

struct Ids{
    rotation: widget::Id,
    scale: widget::Id
}

fn model(app: &App) -> Model {

    app.new_window()
        .with_dimensions(640, 360)
        .view(view)
        .build()
        .unwrap();

    let mut ui = app.new_ui().build().unwrap();

    let ids  = Ids{
        rotation: ui.generate_widget_id(),
        scale: ui.generate_widget_id()
    };

    let filename = "manhattan.geojson";
    let geojson_str = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");
    
    let geojson = geojson_str.parse::<GeoJson>().unwrap();
    let proccessed = process_geo_json(geojson);
    Model {shapes:proccessed, scale: 1.0 , rotation: 0.0, ui:ui, ids:ids}
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    let ui = &mut model.ui.set_widgets();
    
    fn slider(val: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
        widget::Slider::new(val, min, max)
            .w_h(200.0, 30.0)
            .label_font_size(15)
            .rgb(0.3, 0.3, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.0)
    }
    
    for value in slider(model.rotation,0.0,180.0)
        .label("rotation")
        .set(model.ids.rotation,ui){

            model.rotation = value
        }
    
    for value in slider(model.scale,0.0,100.0)
        .label("scale")
        .set(model.ids.scale,ui){
            model.scale=value
        }
}

fn match_geometry(geom: Geometry) -> Option<Vec<Point3<f32>>> {
    match geom.value {
        Value::Polygon(_) => {
            let poly: Polygon<f32> = geom.value.try_into().expect("Unable to convert poly");
            let coords: Vec<Point3<f32>> = poly.exterior().clone()
                .into_iter()
                .map(|point| pt3(point.x, point.y,0.0))
                .collect();
            return Some(coords)
        },
        Value::MultiPolygon(_)=>{
            let poly: MultiPolygon<f32> = geom.value.try_into().expect("Unable to convert poly");
            let coords: Vec<Point3<f32>> = poly.into_iter().next().unwrap().exterior().clone()
                .into_iter()
                .map(|point| pt3(point.x, point.y,0.0))
                .collect();
            return Some(coords)
        }
        _ => 
            return None
    };
}

fn process_geo_json(gj: GeoJson) -> Vec<Option<Vec<Point3<f32>>>> {
    match gj {
        GeoJson::FeatureCollection(collection) => collection
            .features
            .into_iter()
            .filter_map(|feature| feature.geometry)
            .map(match_geometry)
            .collect(),
        GeoJson::Feature(feature) => {
            match feature.geometry{
                Some(geom) => vec![match_geometry(geom)],
                None => panic!("no geomery")
            }
        }
        GeoJson::Geometry(geometry) => vec![match_geometry(geometry)],
    }
}

fn calc_extent(shapes: &Vec<Option<Vec<Point3<f32>>>>) -> Vec<f32>{
    let mut max_x = -1000000.0;
    let mut min_x = 100000.0;
    let mut max_y = -1000000.0;
    let mut min_y = 100000.0;

    for shape in shapes.into_iter().cloned(){
        for point in shape.unwrap().into_iter(){
            if point.x > max_x{
                max_x=point.x;
            }
            if point.x < min_x {
                min_x=point.x
            }
            if point.y > max_y {
                max_y=point.y;
            }
            if point.y < min_y {
                min_y=point.y
            }
        }
    };
    return vec![
        min_x,max_x,min_y,max_y 
    ]
}

fn draw_poly(shape: &Vec<Point3<f32>>, extent: &Vec<f32>, draw: &Draw , win: &nannou::geom::Rect){

    let scaled_points: Vec<Point3> = shape.into_iter()
                            .cloned()
                            .map(|p| pt3( 
                                (p.x - extent[0])*win.w()/(extent[1]-extent[0]) ,
                                (p.y - extent[2])*win.h()/(extent[3]-extent[2]) ,
                                 0.0 )).collect();
  
    draw.polygon()
        .points(scaled_points.clone())
        .color(BLUE)
        .x(-win.w()/2.0)
        .y(-win.h()/2.0);
        
    let half_thickness= 0.5;

    draw.polyline().vertices(half_thickness, 
        scaled_points.clone().into_iter().map(|p|{
                let rgba = nannou::color::Rgba::new(0.0, 0.0, 0.0, 1.0);
                geom::vertex::Rgba(p, rgba)
            }
        ))
        .x(-win.w()/2.0)
        .y(-win.h()/2.0);
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {

    let draw = app.draw();
    let win = app.window_rect();
    frame.clear(BLACK);

    let extent = calc_extent(&model.shapes);
    for shape in model.shapes.iter(){
        match shape{
            Some(shape)=> {
                draw_poly(shape,&extent,&draw, &win);
            },
            None => println!("No shape")
        }
    }

    draw.to_frame(app, &frame).unwrap();
    model.ui.draw_to_frame(app, &frame).unwrap();

    frame
}

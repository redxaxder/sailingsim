use bevy::{
    prelude::*,
    input::keyboard::KeyboardInput,
};

use std::fmt::Display;
use std::fmt;

mod dir;
mod vector;

use dir::Dir;
use vector::V;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_event::<Action>()
        .add_system(get_movement_input.system().label("input"))
        .add_system_set(
            SystemSet::new()
                .with_system(move_ship.system())
                .after("input")
        )
        .add_system(update_heading.system())
        .add_system(update_display_position.system())
        .run()
}

const TILE_SIZE: f32 = 30.0;

#[derive(Clone,Copy,Eq,PartialEq,Hash,Debug)]
pub struct Position(V);
pub struct Player;

#[derive(Clone,Copy,Eq,PartialEq,Hash,Debug)]
pub struct Heading(pub Dir);

impl Heading {
    pub fn interpolate(self, rhs: Heading) -> Option<Vec<Heading>> {
        let r:Vec<Heading> =
              self.0.interpolate(rhs.0)?
              .drain(..)
              .map(Heading).collect();
        Some(r)
    }
}


#[derive(Clone,Copy,Eq,PartialEq,Hash,Debug)]
pub struct Wind(Dir);

#[derive(Clone,Copy,Eq,PartialEq,Hash,Debug,PartialOrd,Ord)]
enum PointOfSail {
    Running,
    BroadReach,
    BeamReach,
    CloseHauled,
    InIrons,
}

impl Display for PointOfSail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PointOfSail::Running => "running",
            PointOfSail::BroadReach => "broad reach",
            PointOfSail::BeamReach =>  "beam reach",
            PointOfSail::CloseHauled => "close hauled",
            PointOfSail::InIrons =>  "in irons",
        };
        write!(f, "{}", s)
    }
}

impl PointOfSail {
    pub fn pos(h: Heading, w: Wind) -> PointOfSail {
        match (w.0 - h.0).u8() {
            0 => PointOfSail::Running,
            1 => PointOfSail::BroadReach,
            2 => PointOfSail::BeamReach,
            3 => PointOfSail::CloseHauled,
            4 => PointOfSail::InIrons,
            5 => PointOfSail::CloseHauled,
            6 => PointOfSail::BeamReach,
            7 => PointOfSail::BroadReach,
            _ => panic!("impossible direction"),
        }
    }

    pub fn cost(self) -> u8 {
        match self {
            PointOfSail::Running     => 0,
            PointOfSail::BroadReach  => 1,
            PointOfSail::BeamReach   => 2,
            PointOfSail::CloseHauled => 4,
            PointOfSail::InIrons     => 8,
        }
    }
}

#[derive(Clone,Copy,Eq,PartialEq,Hash,Debug,PartialOrd,Ord)]
pub struct Maneuver(pub u8);

impl Maneuver {
    const MAX: Maneuver = Maneuver(10);

    pub fn cost(w: Wind, h1: Heading, h2: Heading) -> u8 {
        use itertools::Itertools;
        let intermediate_headings = h1.interpolate(h2);
        match intermediate_headings {
            None => u8::MAX, // cannot reverse direction!
            Some(mut hs) => {
                let mut cost = 0;
                let poss: Vec<PointOfSail> = hs.drain(..)
                    .map(|h| { PointOfSail::pos(h,w) })
                    .collect();
                for (&prev,&next) in poss.iter().tuple_windows() {
                    cost += Maneuver::cost_1(prev, next);
                }
                cost
            },
        }
    }

    fn cost_1(prev: PointOfSail, next: PointOfSail) -> u8 {
        use std::cmp::Ordering;
        match prev.cmp(&next) {
            Ordering::Equal => 0,
            Ordering::Greater => 1,
            Ordering::Less => next.cost(),
        }
    }
}

fn move_ship(
    mut query: Query<(&mut Position, &mut Heading, &mut Maneuver), With<Player>>,
    wind_q: Query<&Wind>,
    mut e: EventReader<Action>
    ) {
    if let
        (Ok((mut p, mut h, mut m)),
         Ok(&wind)
         ) = (query.single_mut(), wind_q.single())
    {
        let pos = PointOfSail::pos(*h,wind);
        for &action in e.iter() {
            let target:Dir = match action {
                Action::Wait => h.0,
                Action::Move(dir) => dir,
            };
            let cost = Maneuver::cost(wind, *h, Heading(target));

            // only do anything if derired action is affordable
            if cost <= m.0 {
                m.0 -= cost; // spend the maneuver cost
                if cost == 0 { // if no maneuver cost, recover a point
                    m.0 = (m.0 + 1).min(10);
                }
                h.0 = target; // set heading to new one
                if pos != PointOfSail::InIrons {
                    p.0 = p.0 + h.0.v(); // advance ship
                }
            }
        }
    }
}

fn update_display_position(
    mut query: Query<(&mut Transform, &Position), Changed<Position>>
    ) {
    for (mut t, pos) in query.iter_mut() {
        t.translation.x = (pos.0.x as f32) * TILE_SIZE;
        t.translation.y = (pos.0.y as f32) * TILE_SIZE;
    }
}

fn update_heading(
    heading_query: Query<(&Heading, &Maneuver),
                         (With<Player>, Or<(Changed<Heading>, Changed<Maneuver>)>)>,
    wind_query: Query<&Wind>,
    mut text_query: Query<&mut Text>,
){
    if let Ok(&wind) = wind_query.single() {
        if let Ok((&heading, &maneuver)) = heading_query.single() {
            if let Ok(mut text) = text_query.single_mut() {
                let pos = PointOfSail::pos(heading,wind);
                let s = format!("Wind: {}  Heading: {}\nPoint of sail: {}\nManeuver: {} / {}",
                    wind.0, heading.0, pos, maneuver.0, Maneuver::MAX.0);
                text.sections[0].value = s;
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ) {

    // Camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // Ship
    let texture_handle = asset_server.load("images/sailboat.png");
    commands.spawn_bundle(
        SpriteBundle {
            material: materials.add(texture_handle.into()),
            transform: Transform::from_xyz(0.0, -111.0, 0.0),
            sprite: Sprite::new(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..Default::default()
        }
    )
        .insert(Position(V{x:0,y:0}))
        .insert(Maneuver::MAX)
        .insert(Player)
        .insert(Heading(Dir::RIGHT))
        ;

    // Wind
    commands.spawn()
        .insert(Wind(Dir::RIGHT))
        ;

    // Heading Display
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Heading: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 30.0,
                        color: Color::rgb(0.5, 0.5, 1.0),
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
}

#[derive(Clone,Copy,Eq,PartialEq,Hash,Debug)]
enum Action{
   Move(Dir),
   Wait
}

fn get_movement_input(mut key_evr: EventReader<KeyboardInput>, mut w: EventWriter<Action>) {
    use bevy::input::ElementState;
    for ev in key_evr.iter() {
        if let (ElementState::Pressed, Some(code)) = (ev.state, ev.key_code) {
            match code {
                KeyCode::Right | KeyCode::D | KeyCode::L =>
                    w.send(Action::Move(Dir::RIGHT)),
                KeyCode::Left | KeyCode::A | KeyCode::H =>
                    w.send(Action::Move(Dir::LEFT)),
                KeyCode::Up | KeyCode::W | KeyCode::K =>
                    w.send(Action::Move(Dir::UP)),
                KeyCode::Down | KeyCode::S | KeyCode::J =>
                    w.send(Action::Move(Dir::DOWN)),
               KeyCode::Q | KeyCode::Y =>
                    w.send(Action::Move(Dir::UPLEFT)),
               KeyCode::E | KeyCode::U =>
                    w.send(Action::Move(Dir::UPRIGHT)),
               KeyCode::Z | KeyCode::B =>
                    w.send(Action::Move(Dir::DOWNLEFT)),
               KeyCode::C | KeyCode::N =>
                    w.send(Action::Move(Dir::DOWNRIGHT)),
               KeyCode::Space =>
                    w.send(Action::Wait),
                _ => {}
            }
        }
    }
}

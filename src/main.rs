use anyhow::{bail, Context, Result};
use counter::Counter;
use itertools::iproduct;
use lazy_static::lazy_static;
use log::{debug, info};
use rustmatica::{util::Vec3, BlockState, Litematic, Region};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet, VecDeque},
    env,
    error::Error,
    ops::Add,
    path::Path,
};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum Direction {
    Up,
    Down,
    North,
    South,
    East,
    West,
}

impl Add<Direction> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Direction) -> Self::Output {
        self + rhs.to_vec3()
    }
}

impl Direction {
    fn to_vec3(&self) -> Vec3 {
        match self {
            Direction::Up => Vec3::new(0, 1, 0),
            Direction::Down => Vec3::new(0, -1, 0),
            Direction::North => Vec3::new(0, 0, -1),
            Direction::South => Vec3::new(0, 0, 1),
            Direction::East => Vec3::new(1, 0, 0),
            Direction::West => Vec3::new(-1, 0, 0),
        }
    }

    fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    fn from_name(name: &str) -> Result<Self> {
        match name {
            "north" => Ok(Self::North),
            "south" => Ok(Self::South),
            "east" => Ok(Self::East),
            "west" => Ok(Self::West),
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            _ => bail!("Can't create a direction from name {}", name),
        }
    }

    // fn from(v: Vec3) -> Result<Self> {
    //     match (v.x, v.y, v.z) {
    //         (0, 1, 0) => Ok(Direction::Up),
    //         (0, -1, 0) => Ok(Direction::Down),
    //         (0, 0, -1) => Ok(Direction::North),
    //         (0, 0, 1) => Ok(Direction::South),
    //         (1, 0, 0) => Ok(Direction::East),
    //         (-1, 0, 0) => Ok(Direction::West),
    //         _ => {
    //             bail!("Can't create a direction from {:?}", v)
    //         }
    //     }
    // }

    fn all() -> [Self; 6] {
        [
            Direction::Up,
            Direction::Down,
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ]
    }
}

// const SOLID_BLOCKS: [&str; 35] = [
//     "minecraft:andesite",
//     "minecraft:blue_concrete",
//     "minecraft:bone_block",
//     "minecraft:calcite",
//     "minecraft:chiseled_quartz_block",
//     "minecraft:cobblestone",
//     "minecraft:copper_block",
//     "minecraft:deepslate_bricks",
//     "minecraft:deepslate_tiles",
//     "minecraft:diorite",
//     "minecraft:dirt",
//     "minecraft:glowstone",
//     "minecraft:gold_block",
//     "minecraft:lapis_block",
//     "minecraft:lime_wool",
//     "minecraft:mushroom_stem",
//     "minecraft:netherrack",
//     "minecraft:oak_wood",
//     "minecraft:ochre_froglight",
//     "minecraft:polished_andesite",
//     "minecraft:polished_diorite",
//     "minecraft:quartz_block",
//     "minecraft:quartz_bricks",
//     "minecraft:quartz_pillar",
//     "minecraft:raw_gold_block",
//     "minecraft:red_nether_bricks",
//     "minecraft:sea_lantern",
//     "minecraft:smooth_quartz",
//     "minecraft:smooth_stone",
//     "minecraft:spruce_wood",
//     "minecraft:stone",
//     "minecraft:stone",
//     "minecraft:stone_bricks",
//     "minecraft:tuff",
//     "minecraft:yellow_glazed_terracotta",
// ];

lazy_static! {
    static ref SOLID_BLOCKS: HashSet<Cow<'static, str>> = HashSet::from(
        [
            "minecraft:andesite",
            "minecraft:blue_concrete",
            "minecraft:bone_block",
            "minecraft:calcite",
            "minecraft:chiseled_quartz_block",
            "minecraft:cobblestone",
            "minecraft:copper_block",
            "minecraft:deepslate_bricks",
            "minecraft:deepslate_tiles",
            "minecraft:diorite",
            "minecraft:dirt",
            "minecraft:glowstone",
            "minecraft:gold_block",
            "minecraft:lapis_block",
            "minecraft:lime_wool",
            "minecraft:mushroom_stem",
            "minecraft:netherrack",
            "minecraft:oak_wood",
            "minecraft:ochre_froglight",
            "minecraft:polished_andesite",
            "minecraft:polished_diorite",
            "minecraft:quartz_block",
            "minecraft:quartz_bricks",
            "minecraft:quartz_pillar",
            "minecraft:raw_gold_block",
            "minecraft:red_nether_bricks",
            "minecraft:sea_lantern",
            "minecraft:smooth_quartz",
            "minecraft:smooth_stone",
            "minecraft:spruce_wood",
            "minecraft:stone",
            "minecraft:stone",
            "minecraft:stone_bricks",
            "minecraft:tuff",
            "minecraft:yellow_glazed_terracotta",
        ]
        .map(Cow::from)
    );
}

fn materials(filename: &str) -> Result<(), Box<dyn Error>> {
    debug!("Reading schematic {}... ", filename);
    let schematic = Litematic::read_file(filename)?;
    debug!("done.");

    // count blocks in first region
    let mut counter = Counter::new();
    for region in schematic.regions.iter() {
        for (_, blockstate) in region.blocks() {
            if blockstate.name == "minecraft:air" {
                continue;
            }
            if blockstate.name.ends_with("_wall_sign") {
                counter[&Cow::from(blockstate.name.replace("_wall_sign", "_sign"))] += 1;
            } else {
                counter[&blockstate.name] += 1;
            }
        }
        // println!("====== entities =======");
        // for x in region.entities.iter() {
        //     dbg!(x);
        // }
        // println!("====== tile_entities =======");
        // for x in region.tile_entities.iter() {
        //     dbg!(x);
        // }
    }

    println!("====== materials =======");
    // sort in reverse, print
    let mut ml: Vec<(&Cow<str>, &i32)> = counter.iter().collect();
    ml.sort_by_key(|(_, v)| -v.to_owned());
    for (k, v) in ml {
        println!("{} {}", k, v);
    }

    Ok(())
}

fn replace(input: &str, output: &str) -> Result<(), Box<dyn Error>> {
    debug!("Reading schematic {}... ", input);
    let mut schematic = Litematic::read_file(input)?;
    debug!("done.");

    let mut output_schematic = Litematic::new(
        Path::new(output)
            .file_name()
            .context("filename required")?
            .to_string_lossy()
            .replace(".litematic", "")
            .into(),
        schematic.description,
        schematic.author,
    );

    // copy the region to the output schema
    for region in schematic.regions.iter_mut() {
        let output_region = region.clone();
        output_schematic.regions.push(output_region);
        let Some(output_region) = output_schematic.regions.last_mut() else {
            continue;
        };

        for (pos, blockstate) in region.blocks() {
            if blockstate.name == "minecraft:lime_wool" {
                output_region.set_block(
                    pos,
                    BlockState {
                        name: Cow::from("minecraft:air"),
                        properties: None,
                    },
                );
            }
        }
    }

    output_schematic.write_file(output)?;
    Ok(())
}

// fn reachable_directions(blockstate: &BlockState) -> HashSet<Direction> {
//     // let solid_blocks = HashSet::from(SOLID_BLOCKS.map(Cow::from));
//
//     if SOLID_BLOCKS.contains(&blockstate.name) {
//         return HashSet::new();
//     }
//
//     // let west = Vec3::new(-1, 0, 0);
//     // let east = Vec3::new(1, 0, 0);
//     // let down = Vec3::new(0, -1, 0);
//     // let up = Vec3::new(0, 1, 0);
//     // let north = Vec3::new(0, 0, -1);
//     // let south = Vec3::new(0, 0, 1);
//     //
//     // let mut dirs = HashSet::from([west, east, down, up, north, south]);
//     let mut dirs = HashSet::from(Direction::all());
//
//     if blockstate.name.ends_with("_stairs") {
//         let Some(props) = &blockstate.properties else {
//             return dirs;
//         };
//
//         let shape = props.get("shape").map_or(String::new(), |c| c.to_string());
//         let half = props.get("half").map_or(String::new(), |c| c.to_string());
//         let facing = props.get("facing").map_or(String::new(), |c| c.to_string());
//
//         if shape == "straight" {
//             match facing.as_str() {
//                 "north" => {
//                     dirs.remove(&Direction::North);
//                 }
//                 "south" => {
//                     dirs.remove(&Direction::South);
//                 }
//                 "east" => {
//                     dirs.remove(&Direction::East);
//                 }
//                 "west" => {
//                     dirs.remove(&Direction::West);
//                 }
//                 _ => {}
//             };
//         }
//         if half == "top" {
//             dirs.remove(&Direction::Up);
//         }
//         if half == "bottom" {
//             dirs.remove(&Direction::Down);
//         }
//     }
//
//     if blockstate.name.ends_with("_slab") {
//         let Some(props) = &blockstate.properties else {
//             return dirs;
//         };
//         let slabtype = props.get("type").map_or(String::new(), |c| c.to_string());
//
//         if slabtype == "double" {
//             return HashSet::new();
//         }
//         if slabtype == "top" {
//             dirs.remove(&Direction::Up);
//         }
//         if slabtype == "bottom" {
//             dirs.remove(&Direction::Down);
//         }
//     }
//
//     dirs
// }

// divide a block shape into 8 sub-blocks
struct BlockShape {
    // [x][y][z]
    // x: 0 = west, 1 = east
    // y: 0 = bottom, 1 = top
    // z: 0 = north, 1 = south
    corners: [[[bool; 2]; 2]; 2],
}

#[derive(PartialEq, Eq)]
struct Corner {
    x: usize,
    y: usize,
    z: usize,
}

impl Corner {
    fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }
}

impl BlockShape {
    fn all_corners() -> Vec<Corner> {
        iproduct!(0..=1, 0..=1, 0..=1)
            .map(|(x, y, z)| Corner::new(x, y, z))
            .collect()
    }

    fn side(dir: &Direction) -> Vec<Corner> {
        let all = Self::all_corners();
        match dir {
            Direction::Up => all.into_iter().filter(|v| v.y == 1).collect(),
            Direction::Down => all.into_iter().filter(|v| v.y == 0).collect(),
            Direction::North => all.into_iter().filter(|v| v.z == 0).collect(),
            Direction::South => all.into_iter().filter(|v| v.z == 1).collect(),
            Direction::East => all.into_iter().filter(|v| v.x == 1).collect(),
            Direction::West => all.into_iter().filter(|v| v.x == 0).collect(),
        }
    }

    fn edge(a: &Direction, b: &Direction) -> Vec<Corner> {
        let side_b = Self::side(b);
        Self::side(a)
            .into_iter()
            .filter(|v| side_b.contains(v))
            .collect()
    }

    fn from_stairs_props(shape: &str, half: &str, facing: &str) -> Self {
        let mut blockshape = Self {
            corners: [[[false; 2]; 2]; 2],
        };

        if half == "top" {
            for c in Self::side(&Direction::Up) {
                blockshape.corners[c.x][c.y][c.z] = true;
            }
        }
        if half == "bottom" {
            for c in Self::side(&Direction::Down) {
                blockshape.corners[c.x][c.y][c.z] = true;
            }
        }
        if shape == "straight" {
            for c in Self::side(&Direction::from_name(facing).unwrap()) {
                blockshape.corners[c.x][c.y][c.z] = true;
            }
        }
        if shape.starts_with("outer_") || shape.starts_with("inner_") {
            let side_a = Direction::from_name(facing).unwrap();
            let (mode, rot) = shape.split_once('_').unwrap();
            let side_b = match (facing, rot) {
                ("north", "right") => Direction::East,
                ("north", "left") => Direction::West,
                ("east", "right") => Direction::South,
                ("east", "left") => Direction::North,
                ("south", "right") => Direction::West,
                ("south", "left") => Direction::East,
                ("west", "right") => Direction::North,
                ("west", "left") => Direction::South,
                _ => {
                    panic!(
                        "Unexpected properties in stairs block facing={} shape={}",
                        facing, shape
                    )
                }
            };
            if mode == "outer" {
                for c in Self::edge(&side_a, &side_b) {
                    blockshape.corners[c.x][c.y][c.z] = true;
                }
            }
            if mode == "inner" {
                for c in Self::side(&side_a) {
                    blockshape.corners[c.x][c.y][c.z] = true;
                }
                for c in Self::side(&side_b) {
                    blockshape.corners[c.x][c.y][c.z] = true;
                }
            }
        }

        blockshape
    }

    fn solid() -> Self {
        Self {
            corners: [[[true; 2]; 2]; 2],
        }
    }

    fn from_slab_props(slabtype: &str) -> Self {
        let mut blockshape = Self {
            corners: [[[false; 2]; 2]; 2],
        };

        if slabtype == "double" {
            return Self::solid();
        }
        if slabtype == "top" {
            for c in Self::side(&Direction::Up) {
                blockshape.corners[c.x][c.y][c.z] = true;
            }
        }
        if slabtype == "bottom" {
            for c in Self::side(&Direction::Down) {
                blockshape.corners[c.x][c.y][c.z] = true;
            }
        }

        blockshape
    }

    fn from(block: &BlockState) -> Self {
        let air = Self {
            corners: [[[false; 2]; 2]; 2],
        };

        if SOLID_BLOCKS.contains(&block.name) {
            return Self::solid();
        }

        if block.name.ends_with("_stairs") {
            let Some(props) = &block.properties else {
                return air;
            };

            let shape = props.get("shape").map_or(String::new(), |c| c.to_string());
            let half = props.get("half").map_or(String::new(), |c| c.to_string());
            let facing = props.get("facing").map_or(String::new(), |c| c.to_string());

            return Self::from_stairs_props(&shape, &half, &facing);
        }

        if block.name.ends_with("_slab") {
            let Some(props) = &block.properties else {
                return air;
            };

            let slabtype = props.get("type").map_or(String::new(), |c| c.to_string());

            return Self::from_slab_props(&slabtype);
        }

        match block.name.as_ref() {
            "minecraft:air" => {}
            "minecraft:campfire" => {}
            "minecraft:fire" => {}
            "minecraft:iron_trapdoor" => {}
            "minecraft:lantern" => {}
            "minecraft:nether_brick_fence" => {}
            "minecraft:observer" => {}
            "minecraft:spruce_trapdoor" => {}
            "minecraft:spruce_wall_sign" => {}
            "minecraft:torch" => {}
            "minecraft:water" => {}

            x if x.ends_with("wall") => {}
            _ => {
                debug!("Don't know the shape of {}", block.name);
            }
        };
        air
    }
}

fn can_move(from: &BlockState, to: &BlockState, dir: &Direction) -> bool {
    let from_shape = BlockShape::from(from);
    let to_shape = BlockShape::from(to);

    let from_bits = BlockShape::side(dir)
        .into_iter()
        .map(|c| from_shape.corners[c.x][c.y][c.z]);

    let to_bits = BlockShape::side(&dir.opposite())
        .into_iter()
        .map(|c| to_shape.corners[c.x][c.y][c.z]);

    if from_bits.zip(to_bits).all(|(a, b)| a || b) {
        return false;
    }

    true
}

fn can_see(from: &BlockState, dir: &Direction) -> bool {
    let from_shape = BlockShape::from(from);

    if BlockShape::side(dir)
        .into_iter()
        .map(|c| from_shape.corners[c.x][c.y][c.z])
        .all(|x| x)
    {
        return false;
    }

    true
}

// struct LookAtResult {
//     can_see: bool,
//     can_move: bool,
// }

//  There's two things we need to determine in our BFS when looking from the
//  current block (`current`) to an adjacent block in a given direction (`next`):
//
//  - can the BFS _see_ `next`? this will be used to determine
//    if "the light touches the block". E.g. if `current` is a bottom
//    slab, the BFS will be able to see `next`, regardless of what
//    block it is; but if `current` is stairs with shape "straight"
//    then whether `next` is visible depends on where the stairs is
//    facing, i.e. north, south, etc.
//
//  - can the BFS _move_ to `next`? a neighboor block might be
//    visible, but the BFS won't "move" to it (i.e. it won't be put in the
//    search queue) because theres no "gap" for the light to pass through;
//    e.g. `current` is a bottom slab and `next` is a top slab; in this
//    case, `next` is visible, but it shouldn't be moved to, otherwise
//    the BFS would "break through the walls".
// fn look(current: Vec3, dir: Direction, region: &Region) -> LookAtResult {
//     let from_block = region.get_block(current);
//     let to = current + dir.clone();
//     let to_block = region.get_block(to);
//
//     let can_see = true;
//     let can_move = true;
//
//     let neither = LookAtResult {
//         can_see: false,
//         can_move: false,
//     };
//
//     if SOLID_BLOCKS.contains(&from_block.name) {
//         return neither;
//     }
//     // if solid_blocks.contains(&to_block.name) {
//     //     return LookAtResult {
//     //         can_see: true,
//     //         can_move: false,
//     //     };
//     // }
//
//     // let mut result = true;
//
//     if from_block.name.ends_with("_stairs") {
//         let Some(props) = &from_block.properties else {
//             return neither;
//         };
//
//         let shape = props.get("shape").map_or(String::new(), |c| c.to_string());
//         let half = props.get("half").map_or(String::new(), |c| c.to_string());
//         let facing = props.get("facing").map_or(String::new(), |c| c.to_string());
//
//         // if shape is not straight, we can see in all sideways directions,
//         // but whether we can move depends on `current`'s and `next`'s shapes,
//         // e.g. and `outer_right` stairs next to a straight stairs that "covers
//         // the 0.5x0.5 space would block movement.
//         if shape == "straight" {
//             match facing.as_str() {
//                 "north" => {
//                     if dir == Direction::North {
//                         return neither;
//                     }
//                 }
//                 "south" => {
//                     if dir == Direction::South {
//                         return neither;
//                     }
//                 }
//                 "east" => {
//                     if dir == Direction::East {
//                         return neither;
//                     }
//                 }
//                 "west" => {
//                     if dir == Direction::West {
//                         return neither;
//                     }
//                 }
//                 _ => {}
//             };
//         }
//         if half == "top" && dir == Direction::Up {
//             return neither;
//         }
//         if half == "bottom" && dir == Direction::Down {
//             return neither;
//         }
//     }
//
//     if from_block.name.ends_with("_slab") {
//         let Some(props) = &from_block.properties else {
//             return neither;
//         };
//         let slabtype = props.get("type").map_or(String::new(), |c| c.to_string());
//
//         if slabtype == "double" {
//             return neither;
//         }
//
//         if slabtype == "top" && dir == Direction::Up {
//             return neither;
//         }
//         if slabtype == "bottom" && dir == Direction::Down {
//             return neither;
//         }
//     }
//
//     LookAtResult { can_see, can_move }
// }

struct Node {
    pos: Vec3,
    gen: usize,
}

fn optimize_region<'a>(
    region: &Region<'a>,
    starting_pos: Vec3,
    rainbow: bool,
    inside: Option<Vec3>,
) -> Result<Region<'a>> {
    let mut output_region = region.clone();

    let mut q: VecDeque<Node> = VecDeque::new();
    q.push_back(Node {
        pos: starting_pos,
        gen: 0,
    });

    let mut visited: HashSet<Vec3> = HashSet::new();
    visited.insert(starting_pos);

    let mut reachable_blocks: HashSet<Vec3> = HashSet::new();

    // let west = Vec3::new(-1, 0, 0);
    // let east = Vec3::new(1, 0, 0);
    // let down = Vec3::new(0, -1, 0);
    // let up = Vec3::new(0, 1, 0);
    // let north = Vec3::new(0, 0, -1);
    // let south = Vec3::new(0, 0, 1);
    //
    // let directions = vec![west, east, down, up, north, south];

    let mut parents = HashMap::new();
    let mut light_leaked = false;

    let mut lastgen = 0;

    'bfs: while !q.is_empty() {
        let Node { pos, gen } = q.pop_front().unwrap();
        let current_block = region.get_block(pos);

        if gen != lastgen {
            dbg!(gen);
            lastgen = gen;
        }

        // for direction in reachable_directions(blockstate) {
        for dir in Direction::all() {
            let next_pos = pos + dir.clone();
            if !region.contains(&next_pos) {
                continue;
            }
            let next_block = region.get_block(next_pos);
            if visited.contains(&next_pos) {
                continue;
            }

            if rainbow && next_block.name == "minecraft:air" {
                let rainbow_block = [
                    "minecraft:red_wool",
                    "minecraft:red_concrete",
                    "minecraft:orange_wool",
                    "minecraft:orange_concrete",
                    "minecraft:yellow_wool",
                    "minecraft:yellow_concrete",
                    "minecraft:lime_wool",
                    "minecraft:lime_concrete",
                    "minecraft:cyan_wool",
                    "minecraft:cyan_concrete",
                    "minecraft:light_blue_wool",
                    "minecraft:light_blue_concrete",
                    "minecraft:blue_wool",
                    "minecraft:blue_concrete",
                    "minecraft:purple_wool",
                    "minecraft:purple_concrete",
                ][gen % 16];
                output_region.set_block(
                    next_pos,
                    BlockState {
                        name: Cow::from(rainbow_block),
                        properties: None,
                    },
                );
            }

            if can_see(current_block, &dir) && next_block.name != "minecraft:air" {
                reachable_blocks.insert(next_pos);
            }
            if pos == starting_pos || can_move(current_block, next_block, &dir) {
                q.push_back(Node {
                    pos: next_pos,
                    gen: gen + 1,
                });
                if let Some(inside) = inside {
                    parents.insert(next_pos, pos);
                    if next_pos == inside {
                        debug!("reached inside from start block");
                        light_leaked = true;
                        break 'bfs;
                    }
                }
                visited.insert(next_pos);
            }
        }
    }

    if light_leaked {
        let mut current = inside.unwrap();
        loop {
            let Some(parent) = parents.get(&current) else {
                break;
            };
            if *parent == current {
                break;
            }
            output_region.set_block(
                current,
                BlockState {
                    name: Cow::from("minecraft:red_wool"),
                    properties: None,
                },
            );
            current = *parent;
        }
        return Ok(output_region);
    }

    for (pos, blockstate) in region.blocks() {
        // blocks on the very edge of the region will be considered reachable
        // (not quite sure about this, though)
        if pos.x == region.max_x()
            || pos.x == region.min_x()
            || pos.y == region.max_y()
            || pos.y == region.min_y()
            || pos.z == region.max_z()
            || pos.z == region.min_z()
        {
            continue;
        }
        if reachable_blocks.contains(&pos) {
            continue;
        }
        if blockstate.name == "minecraft:air" {
            continue;
        }
        debug!("Replacing {} at {:?} with air", blockstate.name, pos);
        output_region.set_block(
            pos,
            BlockState {
                name: Cow::from("minecraft:air"),
                properties: None,
            },
        );
    }
    Ok(output_region)
}

fn optimize(input: &str, starting_block_id: &str, output: &str) -> Result<()> {
    let mut starting_pos = None;
    debug!("Reading schematic {}... ", input);
    let schematic = Litematic::read_file(input)?;
    debug!("done.");

    let mut output_schematic = Litematic::new(
        Path::new(output)
            .file_name()
            .context("filename required")?
            .to_string_lossy()
            .replace(".litematic", "")
            .into(),
        schematic.description,
        schematic.author,
    );

    for region in schematic.regions.iter() {
        for (pos, blockstate) in region.blocks() {
            if blockstate.name == starting_block_id {
                starting_pos = Some(pos);
            }
        }
        let Some(starting_pos) = starting_pos else {
            bail!("Starting block id {} not found in region {}", starting_block_id, region.name);
        };

        let optimized_region =
            optimize_region(region, starting_pos, false, Some(Vec3::new(7, 1, 7)))?;
        output_schematic.regions.push(optimized_region);
    }

    output_schematic.write_file(output)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let command = env::args().nth(1).unwrap();

    if command == "materials" {
        let filename = env::args().nth(2).unwrap();
        materials(&filename)?;
    }

    if command == "replace" {
        let input = env::args().nth(2).unwrap();
        let output = env::args().nth(3).unwrap();
        replace(&input, &output)?;
    }

    if command == "optimize" {
        let input = env::args().nth(2).unwrap();
        let output = env::args().nth(3).unwrap();
        optimize(&input, "minecraft:blue_wool", &output)?;
    }

    Ok(())
}

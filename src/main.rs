use anyhow::{bail, Context, Result};
use counter::Counter;
use log::{debug, info};
use rustmatica::{util::Vec3, BlockState, Litematic, Region};
use std::{
    borrow::Cow,
    collections::{HashSet, VecDeque},
    env,
    error::Error,
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

const SOLID_BLOCKS: [&str; 35] = [
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
];

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
        Cow::from(output.replace(".litematic", "")),
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
            if blockstate.name == "minecraft:waxed_cut_copper_stairs" {
                output_region.set_block(
                    pos,
                    BlockState {
                        name: Cow::from("minecraft:cut_copper_stairs"),
                        properties: blockstate.properties.clone(),
                    },
                );
            }
        }
    }

    output_schematic.write_file(output)?;
    Ok(())
}

fn reachable_directions(blockstate: &BlockState) -> HashSet<Direction> {
    let solid_blocks = HashSet::from(SOLID_BLOCKS.map(Cow::from));

    if solid_blocks.contains(&blockstate.name) {
        return HashSet::new();
    }

    // let west = Vec3::new(-1, 0, 0);
    // let east = Vec3::new(1, 0, 0);
    // let down = Vec3::new(0, -1, 0);
    // let up = Vec3::new(0, 1, 0);
    // let north = Vec3::new(0, 0, -1);
    // let south = Vec3::new(0, 0, 1);
    //
    // let mut dirs = HashSet::from([west, east, down, up, north, south]);
    let mut dirs = HashSet::from(Direction::all());

    if blockstate.name.ends_with("_stairs") {
        let Some(props) = &blockstate.properties else {
            return dirs;
        };

        let shape = props.get("shape").map_or(String::new(), |c| c.to_string());
        let half = props.get("half").map_or(String::new(), |c| c.to_string());
        let facing = props.get("facing").map_or(String::new(), |c| c.to_string());

        if shape == "straight" {
            match facing.as_str() {
                "north" => {
                    dirs.remove(&Direction::North);
                }
                "south" => {
                    dirs.remove(&Direction::South);
                }
                "east" => {
                    dirs.remove(&Direction::East);
                }
                "west" => {
                    dirs.remove(&Direction::West);
                }
                _ => {}
            };
        }
        if half == "top" {
            dirs.remove(&Direction::Up);
        }
        if half == "bottom" {
            dirs.remove(&Direction::Down);
        }
    }

    if blockstate.name.ends_with("_slab") {
        let Some(props) = &blockstate.properties else {
            return dirs;
        };
        let slabtype = props.get("type").map_or(String::new(), |c| c.to_string());

        if slabtype == "top" {
            dirs.remove(&Direction::Up);
        }
        if slabtype == "bottom" {
            dirs.remove(&Direction::Down);
        }
    }

    dirs
}

fn can_move_into(blockstate: &BlockState, from: Direction) -> bool {
    let solid_blocks = HashSet::from(SOLID_BLOCKS.map(Cow::from));

    if solid_blocks.contains(&blockstate.name) {
        return false;
    }

    let mut result = true;

    if blockstate.name.ends_with("_stairs") {
        let Some(props) = &blockstate.properties else {
            return false;
        };

        let shape = props.get("shape").map_or(String::new(), |c| c.to_string());
        let half = props.get("half").map_or(String::new(), |c| c.to_string());
        let facing = props.get("facing").map_or(String::new(), |c| c.to_string());

        if shape == "straight" {
            match facing.as_str() {
                "north" => {
                    // if the stairs are facing north, and movement comes from
                    // the north, then from != North will be false; turning result into false
                    // another way of explaining it: if from != North, we keep result as true
                    // maybe another direction will end up blocking movement, but up to this
                    // point, we can't say.
                    result = result && (from != Direction::North)
                }
                "south" => result = result && (from != Direction::South),
                "east" => result = result && (from != Direction::East),
                "west" => result = result && (from != Direction::West),
                _ => {}
            };
        }
        if half == "top" {
            result = result && (from != Direction::Up)
        }
        if half == "bottom" {
            result = result && (from != Direction::Down)
        }
    }

    if blockstate.name.ends_with("_slab") {
        let Some(props) = &blockstate.properties else {
            return false;
        };
        let slabtype = props.get("type").map_or(String::new(), |c| c.to_string());

        if slabtype == "top" {
            result = result && (from != Direction::Up)
        }
        if slabtype == "bottom" {
            result = result && (from != Direction::Down)
        }
    }

    result
}

fn optimize_region<'a>(region: &Region<'a>, starting_pos: Vec3) -> Result<Region<'a>> {
    let mut output_region = region.clone();

    let mut q: VecDeque<(Vec3, usize)> = VecDeque::new();
    q.push_back((starting_pos, 0));

    let mut visited: HashSet<Vec3> = HashSet::new();

    let mut reachable_blocks: HashSet<Vec3> = HashSet::new();

    // let west = Vec3::new(-1, 0, 0);
    // let east = Vec3::new(1, 0, 0);
    // let down = Vec3::new(0, -1, 0);
    // let up = Vec3::new(0, 1, 0);
    // let north = Vec3::new(0, 0, -1);
    // let south = Vec3::new(0, 0, 1);
    //
    // let directions = vec![west, east, down, up, north, south];

    while !q.is_empty() {
        let (pos, gen) = q.pop_front().unwrap();
        let blockstate = region.get_block(pos);

        for direction in reachable_directions(blockstate) {
            let p = pos + direction.to_vec3();
            if !region.contains(&p) {
                continue;
            }
            let block = region.get_block(p);
            if visited.contains(&p) {
                continue;
            }
            if block.name != "minecraft:air" {
                reachable_blocks.insert(p);
            }
            if block.name == "minecraft:air" {
                let rainbow_block = [
                    "minecraft:red_wool",
                    "minecraft:orange_wool",
                    "minecraft:yellow_wool",
                    "minecraft:lime_wool",
                    "minecraft:cyan_wool",
                    "minecraft:light_blue_wool",
                    "minecraft:blue_wool",
                    "minecraft:purple_wool",
                ][gen % 8];
                // dbg!(pos);
                // dbg!(p);
                // dbg!(gen);
                // dbg!(rainbow_block);
                output_region.set_block(
                    p,
                    BlockState {
                        name: Cow::from(rainbow_block),
                        properties: None,
                    },
                );
            }
            // if p.x == 7 && p.z == 8 {
            //     dbg!(pos);
            //     dbg!(p);
            //     dbg!(block);
            //     dbg!(&direction);
            //     dbg!(can_move_into(block, direction.opposite()));
            // }
            if can_move_into(block, direction.opposite()) {
                q.push_back((p, gen + 1));
                visited.insert(p);
            }
        }
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
        info!("Replacing {} at {:?} with air", blockstate.name, pos);
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

        let optimized_region = optimize_region(region, starting_pos)?;
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

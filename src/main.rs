use counter::Counter;
use log::debug;
use rustmatica::{util::Vec3, BlockState, Litematic};
use std::{borrow::Cow, env, error::Error};

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
            counter[&blockstate.name] += 1;
        }
    }

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
        Cow::from("optimatica-name"),
        Cow::from("optimatica-description"),
        Cow::from("optimatica-author"),
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

    Ok(())
}

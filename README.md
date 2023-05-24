# Optimatica

Optimatica is an optimizer for .litematic files, and a couple of additional
tools for manipulating schematics, like printing a material list, or replacing
blocks.

The optimizer's goal is to trim down some builds that, as a result of clones &
fills in creative mode, end up having many blocks that are invisible, but still
inflate the material list.

The optimization process goes as follow:

 - Add one block to your build that's "outside", i.e. in the area of the build
   that the players would normally be. Pick something that your build doesn't
   use, e.g. `minecraft:blue_wool`

 - Run the optimizer and tell it what block you used.

 - The optimizer will "flood fill" the schematic starting on the block from
   step 1.

 - Any block that is not touched by the virtual flood will be replaced by air.

Right now, the optimizer has a very limited knowledge of block shapes, so it
will probably not optimize many blocks it could've.

# Flood reaches the inside

If the optimizer is not removing blocks that you know shouldn't be reachable from
the starting block, try these options:

- `--rainbow` actually fills up the air in the schematic with a rainbow pattern
  built with wool & concrete blocks, starting at the starting block; load the
  resulting schematic in Minecraft and use Litematica's single-layer render
  mode to see how Optimatica's flood-fill moved from the starting block towards
  the inside of the build. If there's only one or two leaks, this could be
  enough.

- `--inside <block_id>`: before running the optimizer, place another block on
  an area of the build that you consider to be "inside", i.e., unreachable;
  Optimatica will stop the flood fill if it finds the "inside" block and it
  will build the path to it from the starting block. See where that path
  crosses your walls, patch your build, and try optimizing again.

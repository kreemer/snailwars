<?xml version="1.0" encoding="UTF-8"?>
<!--
  Placeholder tileset for hand-drawn Tiled maps.

  Tile 0 (grass) is purely visual, used as the default "Ground" layer
  background. Tiles 1-4 double as both visual swatches and gameplay
  markers: each carries a custom string property `kind` that the game
  reads when parsing the "Logic" layer of a level's .tmx file:

    - tile 1: kind=path  - a cell the snails walk through
    - tile 2: kind=build - a cell where the player may build a tower
    - tile 3: kind=start - the single cell snails spawn at (path start)
    - tile 4: kind=end   - the single cell snails walk to (path end)

  Replace tiles.png with real artwork later; the `kind` properties are
  the only thing gameplay code depends on.
-->
<tileset version="1.10" tiledversion="1.11.0" name="tiles" tilewidth="64" tileheight="64" tilecount="5" columns="5">
 <image source="tiles.png" width="320" height="64"/>
 <tile id="1">
  <properties>
   <property name="kind" value="path"/>
  </properties>
 </tile>
 <tile id="2">
  <properties>
   <property name="kind" value="build"/>
  </properties>
 </tile>
 <tile id="3">
  <properties>
   <property name="kind" value="start"/>
  </properties>
 </tile>
 <tile id="4">
  <properties>
   <property name="kind" value="end"/>
  </properties>
 </tile>
</tileset>

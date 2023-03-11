# TODO:

Add scent system
Refactor all (world, res) to take state instead
Remove referneces to res and instead ask state for objects
Make an easy way to translate between Point and (i32, i32)
Make use of i32 and usize more consistent?
Autoexplore sometimes doesn't explore unrevealed wall tiles
Figure out why a lot of entities slows things down so much
Make autorun more responsive to commands
BIG refactor to make an engine module and move all game-mode-specific stuff outside module
Add debugging game modes, for example arena with controls to spawn monsters, item room, entity communication testing, etc
Add a global range fireball button for sim mode

# PNW thing notes

Game idea based on the indigineous cultures of the PNW
Procedurally generate a coastline. Forests have some hazard that stops spread into them. Rivers are generated with different qualities, maybe different kinds of fish
Build a village with houses along the water for population and other buildings that specialize in things.
Objective is to expand through trade, warfare, or espionage
Use gift-giving ceremony somehow
Low specialization of pop? 
Interactions happen through real unit groups like in M&B. Lays a lot of framework for TRENCHES.
Woodworking is major element of industry. Areas can be burned or eliminated to get more berries. Lays framework for FPAC
Commands/designation happens through buildings. Maybe make a building targeting mode to make this easier (L to target tiles, K to target buildings and arrows move to closest building in that direction. Could make for hard to reach buildings).
Basic loop: Start with 10 houses + villagers, chief's cabin, lumber mill, fishing stuff. Build war camp, shell jeweler, fortifications, meeting hall, quarry, food storage. Send trade caravans, war parties, hunting parties. Evolve it from there.

Implementation
- [x] Multi-tile entities
- Z-levels, ground entities, water entities, air entities?, 
- [x] More levels of intent. Designation entites, goal components for AI, knowledge components for AI and player
- Group pathfinding and possible optimizations
- History simulation for testing and generation
- Challenging UI paradigms - maybe look to cogmind for inspiration

- Starting simulation steps: 
[x] Generate fully built village in static map.
Make AI that fishes, cleans, chops wood, collects shells, and sacrifices excess recourses (holds potlatches) to get points. 
Add more advanced controls and UI for zoom, look, time control, etc. 
Diversify resources enough to make trade necessary, add berries to deforested areas, different fish types, diet indicator that tells you if you're not getting enough fat or protein?, general component of health on people used for growth multiplier. (Use culture video at this point to add ecological detail)
Add chieftans, morale and other human traits, and actions beyond work such as eating and sleeping.
Make villages expand. 
Generate three villages in static map. 
Villages start sending trade caravans. 
Add combat units and bandits to test their strength. 
Add war parties and villages go to war sometimes, sue for peace if resources run low and get obliterated if they don't have enough. 
Add slave-taking, slaves, social status (What does it do?)...Somehow integrate free people disobeying chief vs. slaves being coerced and chance to run away?.
Add seasons, food storage, and times with low food availability.
Add fire, make AI use buckets to put out fire, then experiment with fire in combat. FPAC!


FPAC idea: Making a strong enough fire summons a chaos demon. The more live villagers offered, the further back village will be erased from time, making your village more prosperous or something


Great talk on influence maps https://www.gdcvault.com/play/1025243/Spatial-Knowledge-Representation-through-Modular

https://www.youtube.com/watch?v=GSSX0Bc3Mvs
PNW culture - https://www.youtube.com/watch?v=It4AiOLrQhs&t=0s





Note on infinite axis utility system - Just multiplying scores punishes more considerations. Need a compensation factor, They use complicated shit but I think average should be fine?

Give an axis a name field


Modular influence maps


Implement some FPAC levels with WFC https://github.com/mxgmn/WaveFunctionCollapse

# Components to AI system:
Decision making - how an AI decides high-level goal
Action sequencing - What sequence of actions is required to achieve goal
Pathfinding - how to go about specific step

# ideas: 
"smart objects" - objects store how they can be interacted with
central planner - chief in game. Holds stats on the value of actions, so if there's a lot of food stockpiled, fishing has a low value
[x] knowledge graph - entities shouldn't have access to whole map, but need access to more than just their viewshed
current goal - entities should track current goal. Should include: actions to get there, actions achieved?, momentum
method to exchange knowledge graphs
Add a way for entity to ask another entity for knowledge (Do you know where I can find some trees?)


# implementation
Entites need to know: goal, tasks needed to complete goal, current task, momentum

EntityIntent: Goal, momentum, value (used with momentum for recalculating. At first no recalc)

Goal: [Task], current task, knowledge needed

example goals:
chop down tree
fish
clean fish
process lumber

Fish:
Move to water
Try get fish
Put fish in inventory
Bring fish to fish cleaning house
Knowledge pre-req: Know where water is, know where fish cleaner is

Task: 
fn is_complete

Example tasks:
get item
use item
move item

Use(entity, target): ie use axe on tree or use fishing pole on water

enum Target: entity, point




# other shit
May need heirarchical pathfinding at some point

Maybe implement actions for some component types - could mean a components module is needed



https://old.reddit.com/r/roguelikedev/comments/3b4wx2/faq_friday_15_ai/
"A Smart Object can consist of multiple Interactions, and an Interaction holds an Action Chain, which basically guides an actor through a chain of actions required for the interaction (rather than the actor "knowing" how to interact with everything, the Smart Object tells the actor how it should be interacted with)."
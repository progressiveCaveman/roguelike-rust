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

# IAUS notes

Consideration components:
- Name
- Input
- Params (type, m, k, b, c)
- Input params (min, max, tags, etc)

Input params:
- max and min
- tag/status (only when I have invis)
- Ref to other actions (Only 5 secs after X)

Decisions:
Actions linked to a code function (execute skill, move to location, etc)

Use weights to set general (implicit) priority - Multiply consideration scores by weight to get final score

Ai contains Decision Makers, which contain Decision Score Evaluators, which contain Considerations and output a Decision



Overview:
Standardize inputs
<!-- define forumla and response curves -->

<!-- Action contains a number of considerations
Consideration has input and parameters -->

<!-- Response curve types
Linear
Quadratic
logisitic
Logit

Paramters - m,k,c,b

Linear/quad: y=m*(x-c)^k + b
m = slope
k = exponent
b = vert shift
c = horiz shift

Logistic: y = (k * (1/(1+1000em^(-1x+c)))) + b
m=slope of inflection
k=vertical size of curve
b=vert shift
c=horiz shift -->

<!-- Response curve class - clamp input and output -->

<!-- Multiply all considerations to get an action score -->

Inputs:
For distance, pick a max value beyond which all distances are the same

<!-- Create a clearing house to get slamped inputs into system -->

Input types:
MyHealth
MySpeed
DistanceFromMe
AllyCount
etc

Switch on input types and return approp value



"An action is any atomic thing that the agent can do. That is, the equivalent of a "button press". In your example,"


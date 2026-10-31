# War Automata

**Name:** Trevor Thompson  
**Email:** tthompso@pdx.edu  
**Project topic:** Emergent behavior through a grid-based battle simulation  
**Repository:** https://github.com/TrevorWT/war_automata

## Project vision

War Automata is a medieval-themed, turn-based battle simulation inspired by cellular automata such as Conway's Game of Life. Two opposing teams occupy a grid, with units represented by colored squares. Each unit follows a small set of rules based on its type and nearby units. Instead of reproducing like cells in Game of Life, units move and fight. The Rust program will display the grid so that a user can watch battles unfold.

The user will place units for both teams on the grid and start a battle. Teams alternate turns: Team A acts, then Team B acts, completing one round. On its team's turn, each surviving unit gets one action: move or attack. A cell can hold at most one unit. Units act one at a time in a consistent order, so a unit can only move into a cell that is empty when it acts. An attack damages an opposing unit in range, and a unit is removed when its health reaches zero.

I am considering unit types such as infantry, ranged units, commanders, and priests who can convert opposing units to their team. Each type will have its own simple rules, and I may add other types as I develop the design. My first goal is to write down the rules and work through edge cases before implementing them. When more than one action or outcome is possible, the rules will resolve it using a priority order I define. A battle ends when one team has no units remaining or a round limit is reached.

The display will distinguish teams and unit types, show the current turn and remaining units, and report the outcome. I plan to include controls to start, pause, step through turns, and reset a battle so the user can examine how it develops.

## Emergent behavior

Each unit makes decisions using a few local rules rather than a central strategy. I want to see whether formations, clustered fights, bottlenecks, or stalemates arise from those interactions. By changing starting positions and the mix of unit types, I can compare how battles develop and which team wins. These outcomes are possibilities to investigate, not guaranteed results.

## Concerns and scope

The main design challenge is keeping the rules simple while making them precise, especially for movement around occupied cells, target selection, and conversion. Writing out the rules and their priorities first should help me identify and resolve edge cases. I will consider whether turn order affects the balance of a battle. If it becomes an issue, possible adjustments include letting the second team choose its starting positions after seeing the first team's placement, or alternating which team acts first each round. I will also need to prevent battles from running indefinitely, which is why I plan to use a round limit. I will keep the initial display and rules simple so that the simulation remains manageable within the quarter. Terrain and automated comparisons of many battles are possible extensions only if time permits.

## Acknowledgment

I used ChatGPT to discuss the project idea and help draft this proposal. I will document any further AI use and other sources in the project writeup.

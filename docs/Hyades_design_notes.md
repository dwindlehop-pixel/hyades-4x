# Hyades — design notes

> Converted from `Hyades.odt` (despite the extension, the file was plain UTF-8
> text, not an OpenDocument archive). This is the original free-form brainstorm:
> the earliest statement of intent, before the specs split out into
> `Hyades_simulation_model.md`, `Hyades_vehicle_roles.md`, and the rest.
>
> **Wording is preserved verbatim.** Only structure was added — the implicit
> section headers became headings, and tab indentation became nested list
> levels. See "Conversion notes" at the end for the two places the source was
> ambiguous.

---

## Who are we?

Galactic civilizations, or the immortal leaders thereof, or sneaky secret
societies thereof, or AI computers thereof

## How do we win?

Out-expand the other players, out-produce the other players, terminate the other
players, or out-politick the other players, or

## Why is this going to be fun?

- Deterministic combat
- Random retreats from losing combat
  - Enemies can cut retreat off because that happens in real space
    - Using either strategy (encirclement) or tactics (faster engines)
- Lots of destruction
- Occasional surprise attacks
- Communication via icons on areas
- Too much to do, not enough time to do it
- Bluff the whole table
- Make interesting decisions every round
- Fleet velocity matters
- Fuckin wolverine style combat: cheap guy fucks up some expensive guys with
  exactly the right counter upgrades
- Bidding???
- Card game is a flywheel attached to a bigger machine
- Playful means of interacting with a complex puzzle
- Building more power every turn
- Secrets only you know
- Easily understood what is good. Only surprises are what other players do.
- Eve Online moon mineral Trade Distribution 2d gaussian
  - Counterbuilds depend upon specific minerals that can be substituted
- Strong base builds depend upon specific minerals that can't be substituted
  - Counters aren't rock/paper/scissors: they're directed (acyclic?) graphs
- Shared upgrade deck for degree of enhancement diminishing returns when other
  people have, increased return for when you have

### Growth model

Here, `Nt+1` refers to the number of power in next turn,

- `Nt` refers to the number of power in current turn,
- `Rm` refers to the maximum rate of growth,
- `K` is the carrying capacity (number of players?)

### (continued)

- Private action deck
- Some minerals can be substituted. Others can't. The list can be upgraded.
- Variable tier card costs so early game stuff still has value late game
- Maybe some non-compounding resource that you can burst spend?
- Spend actions to get additional upgrades
- Not doing a bunch of math: UI tells you which effects can be played this turn
- Fast to lose, 30-45 minutes to win
- Cards improve other cards that haven't been played yet
- Negate or turn back on or counter or soft counter other players' moves.
- Open Gates - some kind of doppleganging
- Start with everyone co-located, but plenty of room to run
- Like a foundational ascension technology for each that gets informed by
  different cards, and sometimes is a win condition
- Some basic action that has zero cost for every card
- Fighting is costly, but very effective
- Fighting can be won with strategy (production, ground held) or tactics (attack
  patterns and counterbuilds)
- Digital battle royale Inis & 878 Vikings
- Clearly signal win conditions
  - Second win conditions
- Very specific counters because of the lack of variation
  - Everything affects the board
  - The board affects events
  - Preemptive counters that epoch-based
- That spaceship equivalent of a CK3Star (SupremeCommander experimental unit)
- Like Underlords duos with trading leading to clear strategy even with
  suboptimal placement
- Modeling of mind necessary for success
- Value dependent on number of cards in color, or number of opponent's cards in
  color, or current board state
- Location on the action tree
- Location on the upgrade tree
- Restrict allowed targets based on presence of pop, presence of fleet
- Cost depends on a specific card being played, for flavor reasons!
- Pop grows to 4 and then stops growing, but now it's much harder to dislodge
- Some protection even when pop is not 4
- Difficult terrain
- Barren terrain
- Weird terrain
- Some cards deliver variable results depending upon how many of that card have
  been played
- Cards that have effects depending on playing some card in the same round, or a
  previous round
- 3 basic minerals, 3 advanced minerals, and boogity
  - Different upgrades for economies, expansions, and war machines based on
    different minerals
  - 3 basics are worth 2 advanced, are worth 1 boogity, are worth 3 fleets
- Hexes cost a variable number of fleets as the game progresses

## Combat

It's gReat!

## Actions

- Disrupt supply chains through warfare - expensive and thorough
- Disrupt supply chains through trade - cheap and less effective
- Disrupt supply chains through diplomacy - easy to counter
- Disrupt supply chains through industry - late to come online, but thorough
- Local ignoring of attack (Jubilee)
- Local extra benefits of attack (not better attack, but collateral damage,)
- Local convincing enemy's forces to join you
- Moving, without or without conflict, locally or globally
- Global extra benefits of attack (not better attack, but invasion, gunboat
  diplomacy)
- Delay a conflict
- Create local spies (not necessarily in your sphere of influence)
- Annihilation, Exhaustion, Attrition, Fabian —
  [List of military strategies and concepts](https://en.wikipedia.org/wiki/List_of_military_strategies_and_concepts)
- Expand breadth first
- Expand depth first
- Expand in a direction
- Expand towards a resource
- Expand towards empty space
- Profit off some location
- Profit off some mineral
- Create some mineral in an area
- Reward the owner of an area in the future
- Create some mineral in areas where some other mineral/asset is held (like
  miners, or converters)
- Global reward based on location of some mineral/asset
- Convert an action to an upgrade
- Convert minerals to upgrades
- Make it extra super hard to dislodge a 4 cube colony
- Prisoner's dilemma trade
- Mandatory trade with ally
- Ally
- Deny movement this turn only
- Supplement an existing force
- Militarize some civilian craft
- Declare non-aggression
- Conditional surrender
- Blockade
- Maginot Line
- Cut retreat off
- Command ships by class
- Exchange improbable quantities at a loss
- Project force that can't be projected
- Unique experimental unit
- Project force really extra far
- Sabotage enemy ships
- Bribe enemy ships into retreating
- Sneak attack/ambush
- Counterintuitive tactics
- Convert enemy ships into friendly
- Hit and run
- Juke stats in one region for this turn
- Global extra movement
- Local extra movement
- Ignore hit results via chaff
- Force enemy retreat, or partial retreat
- Force battle between enemy forces
- Gate from one area to another
- Rebellion locally
- Counter some other action card locally, or for one turn, or globally (for cost)
- Retrofit existing ships
- Drive up the price of a resource on the global market
- Disrupt an existing alliance based on conditions

## Upgrades

- You can use one mineral as supply?
- One mineral cost is halved
- Supply cost is reduced
- Increase income
- Increase income conditionally, based on other income production
- Production bonus for two minerals
- Reduce played card cost of one card type
- Reduce supply cost of one kind of asset
- Better optimal
- Better speed
- More ships
- More armored ships
- Weapons good at damage
- Weapons good at not being spent on many hostiles
- Better alchemy
- Reduce board demands for playing a card type
- Get income conditionally, based on a specific enemy income production
- Non-interstellar force that cannot be projected, just for defense
- Factoryless
- Always produce basic military craft

## Costs

### Minerals

- Cyan
- Magenta
- Yellow

### Superminerals

- Red
- Green
- Blue

---

## Conversion notes

Two places where the source was ambiguous. Both are flagged rather than silently
"fixed" — resolve them however you intended:

1. **Run-together line.** The source read, as a single line:

   > `Moving, without or without conflict, locally or globallyGlobal extra benefits of attack (not better attack, but invasion, gunboat diplomacy)`

   This is almost certainly two separate action ideas whose newline was lost
   (note the missing break before `Global`), so it is rendered above as two
   bullets. The internal `without or without` is left as written; it may have
   been intended as *with or without*.

2. **Unlabeled section break.** The growth-model variable definitions
   (`Nt+1`, `Nt`, `Rm`, `K`) appear mid-stream inside the "Why is this going to
   be fun?" list, with no heading and no equation — only the "Here, … refers to"
   glossary. The variables are the standard logistic-growth form. Headings were
   added around it to keep the surrounding bullet list intact; the equation
   itself was never in this document.

   Note that `K` here is guessed at as "(number of players?)". That question was
   later settled: **K is player-relative by design.**

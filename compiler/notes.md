# Scheduling

`loop` scheduling:

Have standard dependency table.
Start doing ASAP, only care about local (others except interloop?) dependencies. 
Advancing in the loop body increases the program size.
Once scheduled, now that we know II = loop body length, we can check. Repeat process relaxing II until it works.

`loop.pip` scheduling:

At start, calculate II = lowerbound().
Do ASAP again, except we also check the interloop deps since II is known.
Until all the loop instructions are scheduled, we loop from addr = `loop start` to `loop start + II`. Whenever we schedule an instruction, we also record the stage that it is in. We increment the stage counter every time we complete this address loop. This simulates being in different stages like the assignment document says.
If we are ever unable to schedule an instruction, we must increase II, starting over.

for loop.pip: Valid to schedule() = Doesn't conflict at [i][j] & doesn't have inter-loop or local depdendency.

# Register Allocation

useful programming construct: foreach_insn(), define on Slot struct
takes a closure which takes an instruction as a parameter, and then runs it on each slot if there is something there.

Let's also put the `starts` hashmap inside the ScheduledProgram struct.

## non-pipelined

Phase 2: add all the dependencies. We also handle the phase 4 here, where if an instruction uses an undefined register, then we simply allocate a new entry in the hashmap (unused register) and increment the counter.
Phase 3: add the mov instructions. We maintain a vector of all the (mov) instructions we need to add to the schedule and then call the scheduler again. Should work exactly as expected because the starts dictionary is still intact.
be careful though because the indices are wrong.. so maybe we merge it all together at the end so that it's right
or just bb0 + bb1 and then schedule, then we tack on bb2.. those would be wrong too but it doesn't matter at that point

We do the mov instruction at the end of the loop because for an interloop dependency, the BB0 value and BB1 value can be live at the same time. Since we can run the instructions out of order, we might have output dependencies, so the two values get different names, and thus it is possible for the old value to be used after the BB1 produced value.

## pipelined

phase 1: Allocate each register # of stages + 1 in BB1 ( don't do anything with BB0 or 2 for right now?). We don't actually record the number of stages right now, so we should probably save that somewhere. (ScheduledProgram as well probably).
> The longest dependency is simply the one obtained with k = 0?
What does he mean?

phase 2: Loop invariants are produced in BB0. What does he mean by this?
> The loop invariant column of Table 2, alloc_r identifies instructions of Bb1 whose results are invariant. 
So we scan the column for invariant operands and then assign those, instead of assigning by destination?
We will have to access the dependency, and then look up the start in the starts map, then we can write the slot in the bb0.

phase 3: fix the operands in BB1. Loop invariant dependencies were handled last time, we just look up in the hashmap. Local deps we just add the number of stages to the producing register. 
Local deps => We just check the difference in stages between the producer and consumer, and add the offset accordingly (eqn 3).
Interloop deps => Same as local deps except we just add 1 for the stage offset.

phase 4: fix BB0/BB2
all the interloop producing instructions in BB0 need to be corrected according to the stage offset of P
local deps => just get a new one using the non pipelined strat
post dep => {Iteration offset is 0. Stage offset is the distance between the producer and consumer where we assume we are in the last stage of the loop, since we have been doing the loop epilogue at that point.

why is it not just +1 iteration offset? cause rrb is +1.. so the values of interest are now shifted up => NOPE RRB does not go +1, if you look at the flow diagram, in the EC > 0 false case RRB = RRB. }
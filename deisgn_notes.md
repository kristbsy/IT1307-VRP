# Design notes

## Various notes
The distance between different houses do not alter the satisfaction of the demand constraint, but affects the window constraint.


We could find houses with conflicting windows, such that one nurse could only go to one of them.
This would not just check the window, but also travel times between them.

Even if a nurse does not have two conflicting time windows, it does not mean that the solution satisfies the time window constraint, since their timing may be off.

The end of the time window is the latest point when the treatment can be complete, not the latest arrival time.

There is possible to create a partial ordering of the houses such that a house is before another one if it has to be visited before.
Using this and a list of incompatible houses, it would be trivial to repair any solution.


## Representation
This problem is quite difficult to design a genotype representation for since it involves permutations and multiple actors.
The least the representation needs is to have a mapping from genotype to phenotype, but it should also facilitate mutations and crossover.

The solution in general will be multiple lists, each with a path that one nurse should take.

One possible representation would be numbers for each house and separation markers that denotes that the next sequence is for the next nurse.
This would be a pretty good and simple representation.

If possible, it would be great if there was a representation that would map such that constraints can be satisfied also.

## Constraint handling
There are two constraints in this project which is related to the nurses capacity, and the windows for the patients.

I can see two different approaches to constraint handling:
* Indirect approach
* Repair mechanism

### Indirect approach
In this approach, instead of directly handling the constraint, we will instead bake it into the fitness function.
For this to also work well, it would be preferable if the fitness got better the closer the solution was to being valid.

### Repair mechanism
In this approach, when a solution violates a constraint we try to fix it such that the repaired solution is as similar as possible to the invalid one.
This has the benefit that the algorithm will always provide valid solutions instead of spending a lot of time finding valid genotypes.

If the only constraint was the nurses capacity, it would be easy to repair, but since we also have the constraint of treatment windows, things get harder.

By having implemented a partial ordering such that each house comes before another if it has to be visited before.
Since a house comes with a window of the earliest you can come, and the latest you must be done, we can create another value for the latest time a house can be visited.

There are some questions about implementing a partial ordering:

You can just create an ordering based solely on the latest possible visit time, but this isn't neccesarrily all encompassing as in practice, two houses can be equal if their windows are wide engough.

Since this is the case, a more correct ordering would have to be time aware, since two houses could be equal or not depending on the timing beforehand.
We know that all of the different paths of different nurses are independent, this means that this ordering can be run on each list independently.
By saving the departure time of each house, we can use this to determine the ordering of the two next houses.

The simplest way to determine the ordering is probably to only test both configurations, meaning that we check if two time windows can be satisfied, then swap the houses and see if it's still satisfied.
If both configurations are valid, the houses have equal ordering in this case.
If only one configuration is correct, then the first house is ordered before the other one.
If none of the configurations are valid, we did something wrong, as this should have been taken care of by the incompatible list

The incompatible matrix may not be engough, since even if two houses are compatible in isolation, we may need to arrive late to them leading to them becoming incompatible.

This is the only difficult case to repair.
The reason for this is that there are multiple possible ways to repair this, which would lead to very different genomic results.
We could either randomly chose a repair method each time, which could aid in exploration.
If we chose to repair this case, this is probably the best approach.

Another possibility would be to just use a penalty function in this case and let the GA figure it out theirselves.
This is probably the best solution, as there could be a lot of different changes to the configuration that could fix this.

It's hard to say which approach is the best, so it is important to allow the immplementation to be configured to use both just a penalty function, and also a repair mechanism.

## Implementation

Since there are many different ways to use a GA in this problem, it is important to easily be able to swap out different configurations.
The implementation I cose in this project is by using a builder pattern with a strategy pattern.

In this case there would be a trait for all the different stages.

The stages will be:
* Parent selection stage
* Crossover stage
* Mutation stage
* Repair stage
* Fitness evaluation (also with penalty function)
* Survivor selection stage

The builder will take in strategies corresponding to the above stages

### Ordering rules

Two patients are incompatible if it is impossible to serve both on the same route.
To test this, assume that we arrive at one patient earliest possible.
If it is impossible to complete the patient and travel to the other patient in time both ways, they are incompatible.
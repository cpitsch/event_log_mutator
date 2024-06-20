
## Pipeline Configuration
A mutation pipeline can be defined in a toml configuration file, and supplied to the 
CLI with the `pipeline` parameter:

Certain attributes in the configuration file can be overridden by specifying them through
command line arguments:

```sh
event_log_mutator --pipeline my_pipeline_file.toml --input other_input_log.xes.gz
```

Like this, instead of using the input log configured in the toml file, a different event
log is used.

### Standard Pipeline

```toml
# The path to the input file. Can be .xes or .xes.gz
input = "path/to/input_log.xes"
# The path where to store the mutated log. If not supplied, default to 
# ./<input-log-name>_mutated.xes.gz.
output = "path/to/output.xes.gz"
# Gzip the event output logs. Defaults to false
compress_output = true

[pipeline]
# The list of mutations to apply. They will be applied in exactly the order in the file
[[pipeline.mutations]]
type = "EventSwapper"
activity_1 = "a"
activity_2 = "b"
probability = 1.0

[[pipeline.mutations]]
# Retain only the variants (sequences of activities) that have a support of at least 5
# in the event log.
type = "VariantSupportFilter"
num_supporting_cases = 5

[[pipeline.mutations]]
# For each event with the activity "a", increase its service time by the standard standard_deviation
# of the activity "a", with probability 0.5.
type = "ServiceTimeStdShifter"
activity = "a"
probability = 0.5
standard_deviations = 1.0
```

### Parametrized Pipeline
The parametrized pipeline allows you to specify multiple arguments to the mutations, and
create a mutated event log for each combination of parameters.

The event logs are stored as `log.xes(.gz)` in a path where each applied mutator + parameter 
setting is a directory. So, for instance, one of the save paths for the following 
configuration file is 
`pipeline_outputs/VariantSupportFilter_thresh5/ServiceTimeStdShifter_a_p0.2_std0.5/log.xes.gz`.

```toml
# The path to the input file. Can be .xes or .xes.gz
input = "path/to/input_log.xes"
# The root directory in which the event logs are saved. Defaults to `.`.
output = "pipeline_outputs/"
# Gzip the event output logs. Defaults to false
compress_output = true

# The list of mutations to apply. They will be applied in exactly the order in the file
[parametrized_pipeline]
[[parametrized_pipeline.mutations]]
# Retain only the variants (sequences of activities) that have a support of at least 5
# in the event log.
type = "VariantSupportFilter"
num_supporting_cases = 5

[[parametrized_pipeline.mutations]]
# For each event with the activity "a", increase its service time by the standard standard_deviation
# of the activity "a", with probability 0.5.
type="ServiceTimeStdShifter"
activity = "a"
probability = [0.1, 0.2, 0.3, 0.4, 0.5]
standard_deviations = [0.1, 0.2, 0.3, 0.4, 0.5]
```

### Available Mutators
- [ServiceTimeStdShifter](./src/mutators/service_time_std_shifter.rs)
- [VariantSupportFilter](./src/mutators/filters/variant_support_filter.rs)
- [ActivityRemover](./src/mutators/activity_remover.rs)
- [ActivityRenamer](./src/mutators/activity_rename.rs)
- [ConstantActivity](./src/mutators/constant_activity.rs)
- [EventSwapper](./src/mutators/event_swapper.rs)
- [LogBootstrapper](./src/mutators/log_bootstrapper.rs)
- [PartialOrderCreator](./src/mutators/partial_order_creator.rs)
- [AttributeRemover](./src/mutators/attribute_remover.rs)
- [ServiceTimeMultiplier](./src/mutators/service_time_multiplier.rs)

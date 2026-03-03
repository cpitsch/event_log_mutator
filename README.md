<div align="center">
  <h1>Event Log Mutator</h1>
  <a href="https://github.com/cpitsch/event_log_mutator/actions?query=workflow%3ATests"><img src="https://github.com/cpitsch/event_log_mutator/workflows/Tests/badge.svg" alt="Tests Status"></a>
</div>

## Available Mutators
To see the parameters a mutator takes, follow its respective link

- [ActivityRemover](./src/mutators/activity_remover.rs#L10-L25)
- [ActivityRenamer](./src/mutators/activity_renamer.rs#L10-L27)
- [AttributeRemover](./src/mutators/attribute_remover.rs#L7-L13)
- [AttributeRetainer](./src/mutators/attribute_retainer.rs#L9-L15)
- [ConstantActivity](./src/mutators/constant_activity.rs#L10-L25)
- [EventSwapper](./src/mutators/event_swapper.rs#L14-L35)
- [LogSampler](./src/mutators/log_sampler.rs#L10-L24)
- [LogSplitter](./src/mutators/log_splitter.rs#L18-L26)
- [ServiceTimeMultiplier](./src/mutators/service_time_multiplier.rs#L13-L32)
- [ServiceTimeStdShifter](./src/mutators/service_time_std_shifter.rs#L19-L39)
- [SojournStartAdder](./src/mutators/sojourn_start_adder.rs#L9-L14)

### Filters 
- [AttributeFilter](./src/mutators/filters/attribute_value_filter.rs#L176-L184)
- [CaseDurationFilter](./src/mutators/filters/case_duration_filter.rs#L64-L68)
- [EndpointFilter](./src/mutators/filters/endpoint_filter.rs#L13-L22)
- [FollowerFilter](./src/mutators/filters/follower_filter.rs#L13-L25)
- [TraceLengthFilter](./src/mutators/filters/trace_length_filter.rs#L10-L14)
- [VariantSupportFilter](./src/mutators/filters/variant_support_filter.rs#L10-L18)

## Pipeline Configuration
A mutation pipeline can be defined in a TOML configuration file, and executed using the 
CLI in the `pipeline` mode.

Certain attributes in the configuration file can be overridden by specifying them through
command line arguments:

```sh
event_log_mutator pipeline my_pipeline_file.toml --input other_input_log.xes.gz
```

Like this, instead of using the input log configured in the TOML file, a different event
log is used.

### Standard Pipeline

```toml
# The path to the input file. Can be .xes or .xes.gz
input = "path/to/input_log.xes"
# The path where to store the mutated log. If not supplied, default to 
# ./<input-log-name>_mutated.xes.gz.
output = "path/to/output.xes.gz"
# Gzip the event logs. Defaults to false
compress_output = true

[pipeline]
# Seed for reproducibility
seed = 42

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
# For each event with the activity "a", increase its service time by the standard deviation
# of the activity "a", with probability 0.5.
type = "ServiceTimeStdShifter"
activity = "a"
probability = 0.5
standard_deviations = 1.0
```

### Parametrized Pipeline
If you need to apply a pipeline for various settings, you can parametrize the mutators by
providing lists of values instead.
For parametrized pipelines, the output argument specifies the root path to which to save
the generated event logs.

The event logs are stored as `log.xes(.gz)` in a path where each applied mutator + parameter 
setting is a directory. So, for instance, one of the save paths for the following 
configuration file is 
`pipeline_outputs/VariantSupportFilter_thresh5/ServiceTimeStdShifter_a_p0.2_std0.5/log.xes.gz`.

```toml
# The path to the input file. Can be .xes or .xes.gz
input = "path/to/input_log.xes"
# The root directory in which the event logs are saved. Defaults to `.`.
output = "pipeline_outputs/"
# Gzip the event logs. Defaults to false
compress_output = true

[pipeline]
# Seeds for reproducibility; The mutation chain will be applied for each seed
seed = "1..=10"

# The list of mutations to apply. They will be applied in exactly the order in the file
[[pipeline.mutations]]
# Retain only the variants (sequences of activities) that have a support of at least 5
# in the event log.
type = "VariantSupportFilter"
num_supporting_cases = 5

[[pipeline.mutations]]
# For each event with the activity "a", increase its service time by various factors of the 
# standard deviation of the activity "a", with various probabilities.
# This results in 25 different mutation chains from this pipeline file.
type = "ServiceTimeStdShifter"
activity = "a"
# This mutator typically expects a single float. Providing a list instead will parametrize
# the mutator
probability = [0.1, 0.2, 0.3, 0.4, 0.5]
standard_deviations = [0.1, 0.2, 0.3, 0.4, 0.5]
```

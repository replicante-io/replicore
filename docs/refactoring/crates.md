# Crates organisation
Status: PENDING
Reason: Group related crates and keep source tree cleaner.
Blocking something: NO


## Task
Move and rename crates in this repo into logical groups that better reflect
crates roles, responsibilities, or areas of concern.

The desired outcome is approximately described below.
Crates marked as `FUTURE` are POSSIBLE things that will be added and not a commitment.

  * [x] `agent/`: crates directly interacting with agents (NO CHANGE).
    * [x] `client`: agent client crate (NO CHANGE).
  * [ ] `bin/`: crates that ultimatelly become binaries (NEW).
    * [ ] `replicante`: Replicante Core (RENAME).
    * [ ] `replictl`: Replicante Core CLI (RENAME).
  * [x] `ci/`: CI tools (NO CHANGE).
  * [ ] `cluster/`: crates focused on cluster logic (NEW).
    * [ ] `aggregator`: aggregated view and events generator (RENAME).
    * [ ] `discovery`: cluster discovery sybsystem (RENAME).
    * [ ] `fetcher`: cluster state refresh and diff (RENAME).
  * [x] `common/`: crates shared with agents or other projects (NO CHANGE).
  * [x] `devtools/`: development tools and helpers (NO CHANGE).
  * [x] `docs/`: project documentation.
  * [ ] `models/`: crates defining data models only (NEW).
    * [ ] `core`: Replicante Core models (RENAME).
    * `common`: Move `common/models` here??? (UNLIKELY).
  * [ ] `service/`: crates that provide services to replicante core (NEW).
    * [ ] `coordinator`: the process coordinator create (RENAME).
    * [ ] `tasks`: the task subsystem crate (RENAME).
  * [ ] `store/`: crates focused on storing data (NEW).
    * [ ] `cache`: optional caching layer to speed up other stores (FUTURE).
    * [ ] `metrics`|`stats`: store agent/cluster time-series data (FUTURE).
    * [ ] `primary`: the primary store crate currently `data/store` (RENAME).
    * [ ] `view`: store data used to generate API or other views (NEW).
  * [ ] `stream/`: crates to manage event streams (RENAME).
    * [ ] `actions`: actions state and result stream (FUTURE).
    * [ ] `audit`: auditing records streams (FUTURE).
    * [ ] `events`: events streams (RENAME).
    * [ ] `stream`: generic stream logic shared by other stream crates (FUTURE).

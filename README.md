`cwnote`  - Add vertical annotations to CloudWatch dashboards from CLI
A  lightweight Rust CLI tool for automatically inserting **vertical annotations** (version markers, incident markers, deploy events, alarms, etc.) into **AWS CloudWatch dashboards**.

It integrates easily into CI/CD pipelines and can target a single dashboard or a whole set of dashboards sharing a suffix.

### **Features**

- Annotate **single dashboards** or **multiple dashboards by suffix**
- Add fully custom annotations:
    
    - --label (e.g. "version", "incident", "deploy", "alarm")
    - --value (e.g. "1.4.2-commit123", "INC-4435")
    
- Filter **only widgets whose title contains a substring**
- Supports **ISO8601 / RFC3339 timestamps**
- Defaults to **current UTC timestamp**
- --dry-run mode to preview changes
- Uses AWS Rust SDK v1 best practices (aws_config::defaults(BehaviorVersion::latest()))



### **Example Usage**

  

#### **Add a version marker during deployment**

```
cwnote annotate \
  --dashboard MyServiceDashboard \
  --label version \
  --value "$(git describe --tags --long)"
```

#### **Annotate multiple dashboards at once**

```
cwnote annotate \
  --dashboard-suffix MyService- \
  --label deploy \
  --value "release-2025-01-20"
```

#### **Only annotate widgets whose title contains a keyword**

  

Useful if your dashboards have many graphs, but you only want version lines on a specific group:

```
cwnote annotate \
  --dashboard MyServiceDashboard \
  --label version \
  --value "1.9.0" \
  --widget-title-contains "Latency"
```

#### Provide an explicit timestamp

```
cwnote annotate \
  --dashboard MyServiceDashboard \
  --label incident \
  --value "INC-4435: DB outage" \
  --time "2025-01-20T12:00:00Z"
```

#### Dry-run mode

```
cwnote annotate \
  --dashboard-suffix MyService- \
  --label version \
  --value "preview-run" \
  --dry-run
```


### Installation

#### From source

```
git clone https://github.com/your-org/cwnote
cd cwnote
cargo install --path .
```

or build manually:

```
cargo build --release
```

Binary will be at:

```
target/release/cwnote
```



### Authentication & AWS Regions


cwnote uses standard AWS credential resolution.

You may override the region:

```
--region eu-central-1
```



### How It Works


CloudWatch dashboards are JSON documents containing arrays of widgets.

cwnote:

1. Downloads the dashboard JSON using GetDashboard
2. Locates metric widgets
3. Applies optional filters (title substring, suffix matches)
4. Appends an annotation of the form:

```
{
  "label": "version: 1.9.0",
  "value": "2025-01-20T12:34:56Z"
}
```

5. Uploads the updated dashboard via PutDashboard


Multiple annotations stack naturally and are visible as vertical lines on graphs.



### Command Reference

  

#### **annotate**

```
cwnote annotate [OPTIONS]
```

**Options:**

|**Option**|**Description**|
|---|---|
|--dashboard <name>|Annotate a specific dashboard|
|--dashboard-suffix <suffix>|Annotate all dashboards starting with suffix|
|--label <string>|Annotation label (e.g. version, incident, deploy)|
|--value <string>|Annotation text/value|
|--time <ISO8601>|Custom timestamp (default: UTC now)|
|--widget-title-contains <substr>|Only annotate widgets whose title contains substring|
|--region <region>|AWS region override|
|--dry-run|Preview changes only|

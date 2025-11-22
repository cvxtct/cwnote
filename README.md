
### CLI Usage
```Shell
# Version deployment marker
target/release/cvnote annotate \
  --dashboard CBOT \
  --label "foo-test-version" \
  --value "0.0.0-test" \
  --widget-title-contains "Resource"

# Incident marker
cwnove annotate \
  --dashboard-prefix MyService- \
  --label incident \
  --value "INC-1234: DB outage"

# Alarm / event marker on specific widget
cwnote annotate \
  --dashboard MyService \
  --label alarm \
  --value "High latency" \
  --widget-title-contains "Overall Latency"
```
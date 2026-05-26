{{- /*
Release name is pinned to "ahnlich" so the umbrella's hard-coded sub-chart
defaults (ahnlich-ai.db.host=ahnlich-ahnlich-db, tracing endpoints pointing
at ahnlich-tracing-backend) resolve. Users who want a different release name
must override the pinned defaults explicitly; this fail catches the common
case where they did neither.
*/ -}}
{{- define "ahnlich.validateReleaseName" -}}
{{- if ne .Release.Name "ahnlich" -}}
{{- $expectedDbHost := printf "%s-ahnlich-db" .Release.Name -}}
{{- $expectedTracing := printf "http://%s-tracing-backend:4317" .Release.Name -}}
{{- $aiDbHost := index .Values "ahnlich-ai" "db" "host" -}}
{{- $aiTracing := index .Values "ahnlich-ai" "tracing" "otelEndpoint" -}}
{{- $dbTracing := index .Values "ahnlich-db" "tracing" "otelEndpoint" -}}
{{- if eq $aiDbHost "ahnlich-ahnlich-db" -}}
{{- fail (printf "Release name is %q, not \"ahnlich\". The umbrella's default ahnlich-ai.db.host points at \"ahnlich-ahnlich-db\" which will not resolve. Override with --set ahnlich-ai.db.host=%s" .Release.Name $expectedDbHost) -}}
{{- end -}}
{{- if and (index .Values "tracing" "backend" "enabled") (or (eq $aiTracing "http://ahnlich-tracing-backend:4317") (eq $dbTracing "http://ahnlich-tracing-backend:4317")) -}}
{{- fail (printf "Release name is %q, not \"ahnlich\". The tracing backend Service will be %q but the sub-chart otelEndpoints still point at \"http://ahnlich-tracing-backend:4317\". Override ahnlich-db.tracing.otelEndpoint and ahnlich-ai.tracing.otelEndpoint to %s" .Release.Name (printf "%s-tracing-backend" .Release.Name) $expectedTracing) -}}
{{- end -}}
{{- end -}}
{{- end -}}

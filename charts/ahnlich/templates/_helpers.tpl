{{- define "ahnlich.tracingBackendName" -}}
{{- printf "%s-tracing-backend" .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "ahnlich.tracingBackendLabels" -}}
helm.sh/chart: {{ .Chart.Name }}-{{ .Chart.Version | replace "+" "_" }}
app.kubernetes.io/name: tracing-backend
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/component: tracing
{{- end -}}

{{- define "ahnlich.tracingBackendSelectorLabels" -}}
app.kubernetes.io/name: tracing-backend
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

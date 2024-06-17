#!/usr/bin/env bash
# Use this script to wait for a service to be ready

host="$1"
shift
cmd="$@"

until nc -z -v -w30 $host; do
  >&2 echo "Waiting for $host to be available..."
  sleep 1
done

>&2 echo "$host is available"
exec $cmd

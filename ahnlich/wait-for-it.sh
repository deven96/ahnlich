#!/usr/bin/env bash
# Use this script to wait for a service to be ready

host=$1
shift
port=$1
shift
cmd="$@"

until nc -z -v -w30 $host $port; do
  >&2 echo "Waiting for $host:$port to be available..."
  sleep 1
done

>&2 echo "$host:$port is available"
exec $cmd

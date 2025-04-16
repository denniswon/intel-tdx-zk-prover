#!/bin/bash

# Sample curl command to register an agent
curl -X POST \
  http://localhost:8002/api/agent \
  -H "Content-Type: application/json" \
  -d '{
  "agent_name": "Sample Agent",
  "agent_type": "Verification",
  "agent_uri": "http://example.com/agent",
  "agent_description": "This is a sample agent for verification purposes",
  "agent_owner": "0x1234567890abcdef1234567890abcdef12345678",
  "agent_status": "ACTIVE"
}'


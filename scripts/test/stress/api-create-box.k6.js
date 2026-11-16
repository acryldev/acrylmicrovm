// Modified by ACRYL, 2026
import http from 'k6/http'
import { check, sleep } from 'k6'

import { DEFAULT_BOX_IMAGE } from '../image.js'

const BASE_URL = (__ENV.ACRYLMICROVM_API_URL || 'https://api.dev.webboxes.ai').replace(/\/$/, '')
const TOKEN = __ENV.ACRYLMICROVM_TOKEN || __ENV.ACRYLMICROVM_API_KEY || ''
const PREFIX = __ENV.ACRYLMICROVM_PREFIX || ''
const IMAGE = __ENV.ACRYLMICROVM_STRESS_IMAGE || DEFAULT_BOX_IMAGE
const NAME_PREFIX = __ENV.ACRYLMICROVM_STRESS_NAME_PREFIX || 'stress-api-create'
const CLEANUP = (__ENV.ACRYLMICROVM_STRESS_CLEANUP || '1') !== '0'

const RATE_START = Number(__ENV.ACRYLMICROVM_STRESS_START_RATE || 0.05)
const RATE_1 = Number(__ENV.ACRYLMICROVM_STRESS_RATE_1 || 0.1)
const RATE_2 = Number(__ENV.ACRYLMICROVM_STRESS_RATE_2 || 0.2)
const RATE_3 = Number(__ENV.ACRYLMICROVM_STRESS_RATE_3 || 0.5)
const PREALLOCATED_VUS = Number(__ENV.ACRYLMICROVM_STRESS_PREALLOCATED_VUS || 3)
const MAX_VUS = Number(__ENV.ACRYLMICROVM_STRESS_MAX_VUS || 10)
const ITERATIONS = Number(__ENV.ACRYLMICROVM_STRESS_ITERATIONS || 0)

function perMinuteRate(ratePerSecond) {
  return Math.max(1, Math.ceil(ratePerSecond * 60))
}

export const options = {
  scenarios:
    ITERATIONS > 0
      ? {
          api_create_box_once: {
            executor: 'shared-iterations',
            vus: Number(__ENV.ACRYLMICROVM_STRESS_VUS || 1),
            iterations: ITERATIONS,
            maxDuration: __ENV.ACRYLMICROVM_STRESS_MAX_DURATION || '5m',
          },
        }
      : {
          api_create_box: {
            executor: 'ramping-arrival-rate',
            startRate: perMinuteRate(RATE_START),
            timeUnit: '1m',
            preAllocatedVUs: PREALLOCATED_VUS,
            maxVUs: MAX_VUS,
            stages: [
              { duration: __ENV.ACRYLMICROVM_STRESS_STAGE_1 || '1m', target: perMinuteRate(RATE_1) },
              { duration: __ENV.ACRYLMICROVM_STRESS_STAGE_2 || '2m', target: perMinuteRate(RATE_2) },
              { duration: __ENV.ACRYLMICROVM_STRESS_STAGE_3 || '2m', target: perMinuteRate(RATE_3) },
              { duration: __ENV.ACRYLMICROVM_STRESS_RAMP_DOWN || '30s', target: 0 },
            ],
          },
        },
  thresholds: {
    http_req_failed: [`rate<${__ENV.ACRYLMICROVM_STRESS_MAX_FAILURE_RATE || '0.02'}`],
    http_req_duration: [
      `p(95)<${__ENV.ACRYLMICROVM_STRESS_P95_MS || '30000'}`,
      `p(99)<${__ENV.ACRYLMICROVM_STRESS_P99_MS || '45000'}`,
    ],
  },
}

function authParams() {
  return {
    headers: {
      Authorization: `Bearer ${TOKEN}`,
      'Content-Type': 'application/json',
    },
    timeout: __ENV.ACRYLMICROVM_STRESS_CREATE_TIMEOUT || '60s',
  }
}

function nameForIteration() {
  return `${NAME_PREFIX}-${__ENV.K6_CLOUDRUN_INSTANCE_ID || 'local'}-${__VU}-${__ITER}-${Date.now()}`
}

function jsonValue(res, path) {
  try {
    return res.json(path)
  } catch {
    return null
  }
}

export function setup() {
  if (!TOKEN) {
    throw new Error('Set ACRYLMICROVM_TOKEN or ACRYLMICROVM_API_KEY for authenticated API stress checks.')
  }
  if (!PREFIX) {
    throw new Error('Set ACRYLMICROVM_PREFIX to the organization/path prefix returned by `acrylmicrovm auth whoami`.')
  }

  return { baseUrl: BASE_URL, prefix: PREFIX, auth: authParams() }
}

export default function (cfg) {
  const name = nameForIteration()
  const body = JSON.stringify({
    name,
    image: IMAGE,
    cpus: Number(__ENV.ACRYLMICROVM_STRESS_CPUS || 1),
    memory_mib: Number(__ENV.ACRYLMICROVM_STRESS_MEMORY_MIB || 256),
    disk_size_gb: Number(__ENV.ACRYLMICROVM_STRESS_DISK_SIZE_GB || 1),
    detach: true,
  })

  const create = http.post(`${cfg.baseUrl}/v1/${cfg.prefix}/boxes`, body, cfg.auth)
  const createdBoxId = jsonValue(create, 'box_id')
  check(create, {
    'create box 201': (res) => res.status === 201,
    'create returns box_id': () => Boolean(createdBoxId),
  })

  const boxId = createdBoxId || name
  if (create.status === 201 && CLEANUP) {
    const remove = http.del(`${cfg.baseUrl}/v1/${cfg.prefix}/boxes/${boxId}`, null, cfg.auth)
    check(remove, {
      'delete created box 204/404': (res) => res.status === 204 || res.status === 404,
    })
  }

  sleep(Number(__ENV.ACRYLMICROVM_STRESS_SLEEP_SECONDS || 1))
}

import * as core from '@actions/core'
import * as exec from '@actions/exec'
import * as cache from '@actions/cache'
import * as path from 'path'

async function getGitConfigProperty(propertyName: string): Promise<string> {
  const {stdout, stderr} = await exec.getExecOutput('git', [
    'config',
    propertyName
  ])

  if (stderr) {
    core.setFailed('Failed to get git user.name')
  }

  return stdout.trim()
}

async function getCurrentGitAuthor(): Promise<[string, string]> {
  const name = await getGitConfigProperty('user.name')
  const email = await getGitConfigProperty('user.email')

  return [name, email]
}

async function getActionRef(): Promise<string> {
  let ref = ''
  await exec.getExecOutput('git', ['rev-parse', 'HEAD'], {
    cwd: __dirname,
    listeners: {
      stdout: (data: Buffer) => {
        ref += data.toString()
      }
    }
  })

  // return ref.trim()
  return '0.10.0'
}

async function linuxMacInstall(actionRef: string) {
  const rootActionDir = path.dirname(__dirname)
  console.log(rootActionDir)
  const {stdout} = await exec.getExecOutput('ls', ['-la'], {
    cwd: rootActionDir
  })
  const {exitCode, stderr} = await exec.getExecOutput(
    'sh',
    ['install.sh', actionRef],
    {
      cwd: rootActionDir
    }
  )

  if (exitCode !== 0) {
    core.setFailed(`Failed to install tag-track: ${stderr}`)
  }
}

async function windowsInstall(actionRef: string) {
  core.setFailed('TBI')
}

async function setupDownload() {
  const runnerOS = process.env.RUNNER_OS
  const runnerArch = process.env.RUNNER_ARCH
  const actionRef = await getActionRef()
  const cacheKey = `tag-track_download${runnerOS}_${runnerArch}_${actionRef}`

  const cacheHit = await cache.restoreCache(['tag-track-bin'], cacheKey)
  if (!cacheHit) {
    if (runnerOS == 'windows') {
      await windowsInstall(actionRef)
    } else {
      await linuxMacInstall(actionRef)
    }

    await cache.saveCache(['tag-track-bin'], cacheKey)
  }
}

async function run() {
  setupDownload()
}

run()

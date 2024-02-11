import * as core from '@actions/core'
import * as exec from '@actions/exec'
import * as cache from '@actions/cache'

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

async function windowsDownload(scriptUrl: string, actionRef: string) {
  const {stdout, stderr} = await exec.getExecOutput('powershell', [])
}

async function linuxMacDownload(scriptUrl: string, actionRef: string) {
  const {stderr: stderrCurl} = await exec.getExecOutput('curl', [scriptUrl])
  if (stderrCurl) {
    core.setFailed(`Failed to download tag-track: ${stderrCurl}`)
  }

  let {stderr: stderrSh} = await exec.getExecOutput('sh', [
    'install.sh',
    actionRef
  ])
  if (stderrSh) {
    core.setFailed(`Failed to download tag-track: ${stderrSh}`)
  }

  await exec.getExecOutput('mkdir', ['-p', 'tag-track-bin'])
  await exec.getExecOutput('mv', [
    '$HOME/.tag-track/bin/tag-track',
    './tag-track-bin/tag-track'
  ])
}

async function setupDownload() {
  const runnerOS = process.env.RUNNER_OS
  const runnerArch = process.env.RUNNER_ARCH
  const actionRef = await getActionRef()
  const cacheKey = `tag-track_download${runnerOS}_${runnerArch}_${actionRef}`

  const cacheHit = await cache.restoreCache(['tag-track-bin'], cacheKey)
  if (!cacheHit) {
    const scriptUrl = `https://raw.githubusercontent.com/${process.env.GITHUB_REPOSITORY}/${actionRef}/install.sh`

    if (runnerOS == 'windows') {
      await windowsDownload(scriptUrl, actionRef)
    } else {
      await linuxMacDownload(scriptUrl, actionRef)
    }

    await cache.saveCache(['tag-track-bin'], cacheKey)
  }
}

async function run() {
  setupDownload()
}

run()

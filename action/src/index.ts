import * as core from '@actions/core'
import * as exec from '@actions/exec'

async function getCurrentGitAuthor(): Promise<[string, string]> {
  let name = ''
  let email = ''
  await exec.exec('git', ['config', 'user.name'], {
    listeners: {
      stdout: (data: Buffer) => {
        name += data.toString()
      }
    }
  })
  await exec.exec('git', ['config', 'user.email'], {
    listeners: {
      stdout: (data: Buffer) => {
        email += data.toString()
      }
    }
  })
  return [name.trim(), email.trim()]
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

  return ref.trim()
}

async function setupDownloadRun() {
  const runnerOS = process.env.RUNNER_OS
  const runnerArch = process.env.RUNNER_ARCH
  const actionRef = await getActionRef()
  const cacheKey = `tag-track_download${runnerOS}_${runnerArch}_${actionRef}`
  core.setOutput('cache-key', cacheKey)
}

async function run() {
  const [name, email] = await getCurrentGitAuthor()
  core.debug(`Git author name: ${name}`)
  core.debug(`Git author email: ${email}`)

  setupDownloadRun()
}

run()

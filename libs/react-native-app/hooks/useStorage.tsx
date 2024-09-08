import * as React from 'react'

import {
  createContext,
  FC,
  PropsWithChildren,
  useContext, useEffect,
  useState
} from 'react'
import { getData, storeData } from '@/utils/storage'
import { API_TOKEN, BLOCKMESH_URL, EMAIL, PASSWORD, RUN_LIB } from '@/utils/constants'

export type Nav = 'login' | 'register' | 'dashboard'

export interface StorageType {
  email: string;
  api_token: string;
  password: string;
  url: string;
  run_lib: string;
  nav: Nav;
  setEmail: (email: string) => void;
  setApiToken: (api_token: string) => void;
  setUrl: (url: string) => void;
  setPassword: (password: string) => void;
  setNav: (nav: Nav) => void;
  clear: () => void;
}

export const Context = createContext<StorageType>(
  {} as StorageType
)

function initUrl(): string {
  const APP_ENVIRONMENT = process.env.APP_ENVIRONMENT
  if (APP_ENVIRONMENT === undefined || APP_ENVIRONMENT !== 'local') {
    const url = 'https://app.blockmesh.xyz'
    console.log('initUrl', url)
    return url
  } else {
    const url = 'http://localhost:8000'
    console.log('initUrl', url)
    return url
  }
}

export const useStorage =
  (): StorageType => {
    return useContext(Context)
  }

export const StorageProvider: FC<PropsWithChildren<any>> = ({
                                                              children
                                                            }) => {
  const [email, setEmailInternal] = useState('')
  const [api_token, setApiTokenInternal] = useState('')
  const [url, setUrlInternal] = useState(initUrl())
  const [password, setPasswordInternal] = useState('')
  const [run_lib, setRunLibInternal] = useState('')
  const [nav, setNav] = useState('login' as Nav)

  useEffect(() => {
    (async () => {
      const run_lib = await getData(RUN_LIB)
      console.log('run_lib', run_lib)
      if (run_lib) {
        setRunLibInternal(run_lib)
      }
      const e = await getData(EMAIL)
      console.log('email', e)
      if (e) {
        setEmailInternal(e)
      }
      const token = await getData(API_TOKEN)
      console.log('token', token)
      if (token) {
        setApiTokenInternal(token)
      }
      const url = await getData(BLOCKMESH_URL)
      console.log('url', url)
      if (url) {
        setUrlInternal(url)
      }
      const password = await getData(PASSWORD)
      console.log('password', password)
      if (password) {
        setPasswordInternal(password)
      }
    })()
  }, [])

  function clear() {
    try {
      setEmail('')
      setPassword('')
      setApiToken('')
    } catch (e: any) {
      console.error(`clear error`, e)
    }
  }


  function setEmail(email: string) {
    try {
      storeData(EMAIL, email.toLowerCase()).then(() => {
        setEmailInternal(email.toLowerCase())
      }).catch((e) => {
        console.error(`setEmail:: email = '${email} , error = '${e}`)
      })
    } catch (e: any) {
      console.error(`setEmail:: email = '${email} , error = '${e}`)
    }
  }

  function setUrl(url: string) {
    try {
      storeData(BLOCKMESH_URL, url).then(() => {
        setUrlInternal(url)
      }).catch((e) => {
        console.error(`setUrl:: url = '${url} , error = '${e}`)
      })
    } catch (e: any) {
      console.error(`setUrl:: url = '${url} , error = '${e}`)
    }
  }

  function setApiToken(api_token: string) {
    try {
      storeData(API_TOKEN, api_token).then(() => {
        setApiTokenInternal(api_token)
      }).catch((e) => {
        console.error(`setApiToken:: api_token = '${api_token} , error = '${e}`)
      })
    } catch (e: any) {
      console.error(`setApiToken:: api_token = '${api_token} , error = '${e}`)
    }
  }

  function setPassword(password: string) {
    try {
      storeData(PASSWORD, password).then(() => {
        setPasswordInternal(password)
      }).catch((e) => {
        console.error(`setPassword:: password = '${password} , error = '${e}`)
      })
    } catch (e: any) {
      console.error(`setPassword:: password = '${password} , error = '${e}`)
    }
  }


  return (
    <Context.Provider
      value={{
        email,
        nav,
        api_token,
        url,
        run_lib,
        password,
        setEmail,
        setApiToken,
        setUrl,
        setPassword,
        setNav,
        clear
      }}
    >
      {children}
    </Context.Provider>
  )
}

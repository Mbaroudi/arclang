import axios from 'axios'

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4001/api'

export const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
})

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

export interface User {
  id: string
  email: string
  name: string | null
  role: string
  createdAt: string
}

export interface AuthResponse {
  user: User
  token: string
  refreshToken: string
}

export const authApi = {
  register: async (data: { email: string; password: string; name?: string }) => {
    const response = await api.post<AuthResponse>('/auth/register', data)
    localStorage.setItem('token', response.data.token)
    localStorage.setItem('refreshToken', response.data.refreshToken)
    return response.data
  },

  login: async (data: { email: string; password: string }) => {
    const response = await api.post<AuthResponse>('/auth/login', data)
    localStorage.setItem('token', response.data.token)
    localStorage.setItem('refreshToken', response.data.refreshToken)
    return response.data
  },

  logout: async () => {
    await api.post('/auth/logout')
    localStorage.removeItem('token')
    localStorage.removeItem('refreshToken')
  },

  getMe: async () => {
    const response = await api.get<{ user: User }>('/auth/me')
    return response.data.user
  },
}

export const projectsApi = {
  list: async () => {
    const response = await api.get('/projects')
    return response.data.projects
  },

  get: async (id: string) => {
    const response = await api.get(`/projects/${id}`)
    return response.data.project
  },

  create: async (data: { name: string; description?: string; visibility?: string }) => {
    const response = await api.post('/projects', data)
    return response.data.project
  },

  update: async (id: string, data: { name?: string; description?: string; visibility?: string }) => {
    const response = await api.patch(`/projects/${id}`, data)
    return response.data.project
  },

  delete: async (id: string) => {
    await api.delete(`/projects/${id}`)
  },
}

export const filesApi = {
  list: async (projectId: string) => {
    const response = await api.get(`/files/project/${projectId}`)
    return response.data.files
  },

  get: async (id: string) => {
    const response = await api.get(`/files/${id}`)
    return response.data.file
  },

  create: async (projectId: string, data: { name: string; path: string; content: string; language?: string }) => {
    const response = await api.post(`/files?projectId=${projectId}`, data)
    return response.data.file
  },

  update: async (id: string, data: { name?: string; content?: string }) => {
    const response = await api.patch(`/files/${id}`, data)
    return response.data.file
  },

  delete: async (id: string) => {
    await api.delete(`/files/${id}`)
  },
}

export const compileApi = {
  compile: async (data: { code: string; fileId?: string }) => {
    const response = await api.post('/compile', data)
    return response.data
  },

  validate: async (code: string) => {
    const response = await api.post('/compile/validate', { code })
    return response.data
  },

  history: async (fileId: string) => {
    const response = await api.get(`/compile/history/${fileId}`)
    return response.data.compilations
  },
  
  exportCapella: async (code: string) => {
    const response = await api.post('/compile/export/capella', { code }, {
      responseType: 'blob'
    })
    return response.data
  },
  
  exportMermaid: async (code: string) => {
    const response = await api.post('/compile/export/mermaid', { code })
    return response.data.mermaid
  },
  
  exportJSON: async (code: string) => {
    const response = await api.post('/compile/export/json', { code })
    return response.data
  },
}

export const aiApi = {
  generateRequirement: async (description: string, context?: string, enforceSyntax: boolean = true) => {
    const response = await api.post('/ai/generate/requirement', { 
      description, 
      context,
      enforceSyntax,
      syntaxRules: enforceSyntax ? 'arclang-strict' : undefined
    })
    return response.data
  },
  
  generateComponent: async (description: string, context?: string, enforceSyntax: boolean = true) => {
    const response = await api.post('/ai/generate/component-code', { 
      description, 
      context,
      enforceSyntax,
      syntaxRules: enforceSyntax ? 'arclang-strict' : undefined
    })
    return response.data
  },
  
  suggestArchitecture: async (requirements: string, enforceSyntax: boolean = true) => {
    const response = await api.post('/ai/suggest/architecture', { 
      requirements,
      enforceSyntax,
      syntaxRules: enforceSyntax ? 'arclang-strict' : undefined
    })
    return response.data
  },
  
  reviewCode: async (code: string) => {
    const response = await api.post('/ai/review', { code })
    return response.data
  },
  
  validateSyntax: async (code: string) => {
    const response = await api.post('/ai/validate-syntax', { code })
    return response.data
  },
  
  generateRichArchitecture: async (description: string, systemType?: string, complexity: 'simple' | 'medium' | 'rich' = 'rich') => {
    const response = await api.post('/ai/generate/rich-architecture', { 
      description,
      systemType,
      complexity
    })
    return response.data
  },
}

export const safetyApi = {
  checkCompliance: async (code: string, standard: string = 'iso26262') => {
    const response = await api.post('/safety/check', { code, standard })
    return response.data
  },
  
  hazardAnalysis: async (code: string) => {
    const response = await api.post('/safety/hazard-analysis', { code })
    return response.data
  },
}

export const shareApi = {
  createShare: async (code: string, options?: { expiresIn?: number; password?: string }) => {
    const response = await api.post('/share', { code, ...options })
    return response.data
  },
  
  getShare: async (shareId: string, password?: string) => {
    const response = await api.get(`/share/${shareId}`, { params: { password } })
    return response.data
  },
}

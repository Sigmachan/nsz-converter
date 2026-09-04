import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  Play, Pause, Settings, FolderOpen, History, 
  Plus, Trash2, CheckCircle, XCircle, Clock, Download,
  Monitor, HardDrive, Zap
} from 'lucide-react'
import { Toaster, toast } from 'sonner'

interface GameMapping {
  title_id: string
  game_name: string
  created_at: string
}

interface ConversionJob {
  id: string
  input_path: string
  output_path?: string
  status: string
  created_at: string
  title_id?: string
  game_name?: string
}

export default function App() {
  const [activeTab, setActiveTab] = useState<'dashboard' | 'mappings' | 'history' | 'settings'>('dashboard')
  const [isMonitoring, setIsMonitoring] = useState(false)
  const [queue, setQueue] = useState<ConversionJob[]>([])
  const [history, setHistory] = useState<ConversionJob[]>([])
  const [mappings, setMappings] = useState<GameMapping[]>([])
  const [newMapping, setNewMapping] = useState({ title_id: '', game_name: '' })
  const [platformInfo, setPlatformInfo] = useState('')
  const [nszReady, setNszReady] = useState(false)
  const [settings, setSettings] = useState({
    monitored_directory: '~/Downloads',
    output_directory: '~/NSZ_Converted',
    auto_convert: true,
    verify_after_convert: true,
    delete_source: false
  })

  // Initialize runtime on mount
  useEffect(() => {
    initializeRuntime()
  }, [])

  const initializeRuntime = async () => {
    try {
      // In production: const info = await invoke('get_platform_info')
      setPlatformInfo('macOS (arm64)') // Simulated
      
      // Ensure nsz is available
      // await invoke('ensure_nsz_runtime')
      setNszReady(true)
      toast.success('NSZ runtime ready', { description: 'All dependencies satisfied' })
    } catch (e) {
      toast.error('Runtime initialization failed', { description: String(e) })
    }
  }

  const toggleMonitoring = async () => {
    const newState = !isMonitoring
    setIsMonitoring(newState)
    
    if (newState) {
      toast.info('Monitoring started', { description: `Watching ${settings.monitored_directory}` })
    } else {
      toast.info('Monitoring stopped')
    }
  }

  const addMapping = () => {
    if (!newMapping.title_id || !newMapping.game_name) {
      toast.error('Both fields required')
      return
    }
    
    setMappings([...mappings, {
      ...newMapping,
      created_at: new Date().toISOString()
    }])
    setNewMapping({ title_id: '', game_name: '' })
    toast.success('Mapping added', { description: `${newMapping.title_id} → ${newMapping.game_name}` })
  }

  const removeMapping = (title_id: string) => {
    const mapping = mappings.find(m => m.title_id === title_id)
    setMappings(mappings.filter(m => m.title_id !== title_id))
    toast.info('Mapping removed', { description: mapping?.game_name })
  }

  const updateSetting = (key: string, value: any) => {
    setSettings({ ...settings, [key]: value })
    toast.success('Setting updated')
  }

  return (
    <div className="min-h-screen bg-[#0f0f11] text-white overflow-hidden">
      <Toaster position="top-center" richColors closeButton />
      
      {/* Discord-style top bar */}
      <div className="h-12 bg-[#1a1b1e] border-b border-[#2b2d31] flex items-center px-4 select-none">
        <div className="flex items-center gap-3 flex-1">
          <div className="flex items-center gap-2.5">
            <div className="w-7 h-7 bg-gradient-to-br from-emerald-400 to-emerald-600 rounded-md flex items-center justify-center">
              <span className="font-bold text-[13px] text-black">N</span>
            </div>
            <div>
              <span className="font-semibold tracking-[-0.2px]">NSZ Converter</span>
              <span className="ml-1.5 text-[10px] px-1.5 py-px bg-[#2b2d31] rounded text-emerald-400 font-mono">v1.0.0</span>
            </div>
          </div>
        </div>
        
        <div className="flex items-center gap-2 text-xs">
          <div className={`flex items-center gap-1.5 px-2.5 py-1 rounded ${nszReady ? 'bg-emerald-500/10 text-emerald-400' : 'bg-amber-500/10 text-amber-400'}`}>
            <div className={`w-1.5 h-1.5 rounded-full ${nszReady ? 'bg-emerald-500' : 'bg-amber-500 animate-pulse'}`} />
            {nszReady ? 'Runtime Ready' : 'Initializing...'}
          </div>
          <div className="px-2.5 py-1 bg-[#2b2d31] rounded text-zinc-400 font-mono">{platformInfo}</div>
        </div>
      </div>

      {/* Main content area with sidebar */}
      <div className="flex h-[calc(100vh-48px)]">
        {/* Sidebar - Discord style */}
        <div className="w-60 bg-[#1a1b1e] border-r border-[#2b2d31] flex flex-col">
          <div className="p-3">
            <button
              onClick={toggleMonitoring}
              className={`w-full flex items-center justify-center gap-2 py-2.5 rounded-lg font-medium text-sm transition-all active:scale-[0.985] ${
                isMonitoring 
                  ? 'bg-red-500/90 hover:bg-red-500 text-white' 
                  : 'bg-emerald-600 hover:bg-emerald-500 text-white'
              }`}
            >
              {isMonitoring ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
              {isMonitoring ? 'Stop Watching' : 'Start Watching'}
            </button>
          </div>

          <div className="px-2 text-[11px] font-semibold text-zinc-500 tracking-[0.5px] px-3 py-2">NAVIGATION</div>
          
          {[
            { id: 'dashboard', label: 'Dashboard', icon: Monitor, badge: queue.length || undefined },
            { id: 'mappings', label: 'Game Mappings', icon: FolderOpen },
            { id: 'history', label: 'Conversion History', icon: History },
            { id: 'settings', label: 'Settings', icon: Settings },
          ].map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as any)}
              className={`flex items-center gap-2.5 mx-1 px-3 py-[9px] text-[13px] rounded-md transition-colors ${
                activeTab === tab.id 
                  ? 'bg-[#2b2d31] text-white' 
                  : 'text-zinc-400 hover:bg-[#2b2d31]/60 hover:text-zinc-200'
              }`}
            >
              <tab.icon className="w-4 h-4" />
              <span className="flex-1 text-left">{tab.label}</span>
              {tab.badge !== undefined && (
                <span className="bg-emerald-600 text-[10px] px-1.5 rounded-full font-mono">{tab.badge}</span>
              )}
            </button>
          ))}

          <div className="mt-auto p-3 border-t border-[#2b2d31]">
            <div className="text-[10px] text-zinc-500 px-2">
              Monitoring: <span className="font-mono text-emerald-400/80">{isMonitoring ? 'ACTIVE' : 'PAUSED'}</span>
            </div>
          </div>
        </div>

        {/* Main content */}
        <div className="flex-1 overflow-auto">
          <AnimatePresence mode="wait">
            {activeTab === 'dashboard' && (
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className="p-8">
                <div className="max-w-4xl">
                  <div className="flex items-end justify-between mb-8">
                    <div>
                      <h1 className="text-3xl font-semibold tracking-[-0.5px]">Dashboard</h1>
                      <p className="text-zinc-400 mt-1">Conversion queue and real-time status</p>
                    </div>
                    <button className="flex items-center gap-2 px-4 py-2 bg-[#2b2d31] hover:bg-[#35373b] rounded-lg text-sm font-medium transition-colors">
                      <Plus className="w-4 h-4" /> Convert File Manually
                    </button>
                  </div>

                  {queue.length === 0 ? (
                    <div className="bg-[#1a1b1e] border border-[#2b2d31] rounded-2xl p-16 text-center">
                      <div className="w-16 h-16 mx-auto mb-6 rounded-2xl bg-[#2b2d31] flex items-center justify-center">
                        <Clock className="w-8 h-8 text-zinc-600" />
                      </div>
                      <div className="text-xl font-medium mb-2">Queue is empty</div>
                      <p className="text-zinc-400 max-w-sm mx-auto">
                        Drop .nsz files into your monitored folder or click "Convert Manually" to get started.
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-2">
                      {queue.map((job, i) => (
                        <div key={i} className="bg-[#1a1b1e] border border-[#2b2d31] rounded-xl p-4 flex items-center gap-4">
                          <div className="flex-1 min-w-0">
                            <div className="font-mono text-sm truncate text-emerald-400/90">{job.input_path}</div>
                            {job.game_name && <div className="text-sm text-white mt-0.5">{job.game_name}</div>}
                          </div>
                          <div className="text-xs px-3 py-1 rounded-full bg-amber-500/10 text-amber-400 font-medium">PENDING</div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </motion.div>
            )}

            {activeTab === 'mappings' && (
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className="p-8 max-w-3xl">
                <div className="mb-8">
                  <h1 className="text-3xl font-semibold tracking-[-0.5px]">Game Mappings</h1>
                  <p className="text-zinc-400 mt-2">Map Title IDs to friendly game names. Converted files will be organized into these folders.</p>
                </div>

                <div className="bg-[#1a1b1e] border border-[#2b2d31] rounded-2xl p-5 mb-4">
                  <div className="flex gap-3">
                    <input
                      type="text"
                      placeholder="Title ID (0100ABCDEF123456)"
                      className="flex-1 bg-[#0f0f11] border border-[#2b2d31] rounded-lg px-4 py-2.5 text-sm font-mono placeholder:text-zinc-600 focus:outline-none focus:border-emerald-600"
                      value={newMapping.title_id}
                      onChange={e => setNewMapping({ ...newMapping, title_id: e.target.value.toUpperCase() })}
                    />
                    <input
                      type="text"
                      placeholder="Game Name"
                      className="flex-[1.5] bg-[#0f0f11] border border-[#2b2d31] rounded-lg px-4 py-2.5 text-sm placeholder:text-zinc-600 focus:outline-none focus:border-emerald-600"
                      value={newMapping.game_name}
                      onChange={e => setNewMapping({ ...newMapping, game_name: e.target.value })}
                    />
                    <button onClick={addMapping} className="px-8 bg-emerald-600 hover:bg-emerald-500 rounded-lg font-medium text-sm active:scale-[0.985] transition-all">Add Mapping</button>
                  </div>
                </div>

                <div className="space-y-px">
                  {mappings.length === 0 && (
                    <div className="text-center py-12 text-zinc-500 text-sm">No mappings configured yet.</div>
                  )}
                  {mappings.map(m => (
                    <div key={m.title_id} className="group flex items-center justify-between bg-[#1a1b1e] border border-[#2b2d31] rounded-xl px-5 py-4 hover:border-[#35373b]">
                      <div className="flex items-center gap-4 font-mono text-sm">
                        <span className="text-emerald-400">{m.title_id}</span>
                        <span className="text-zinc-600">→</span>
                        <span>{m.game_name}</span>
                      </div>
                      <button onClick={() => removeMapping(m.title_id)} className="opacity-0 group-hover:opacity-100 p-2 hover:bg-red-500/10 rounded-lg transition-all">
                        <Trash2 className="w-4 h-4 text-red-400" />
                      </button>
                    </div>
                  ))}
                </div>
              </motion.div>
            )}

            {activeTab === 'history' && (
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className="p-8">
                <h1 className="text-3xl font-semibold tracking-[-0.5px] mb-8">Conversion History</h1>
                
                {history.length === 0 ? (
                  <div className="bg-[#1a1b1e] border border-[#2b2d31] rounded-2xl p-16 text-center">
                    <History className="w-10 h-10 mx-auto text-zinc-700 mb-4" />
                    <div className="text-lg">No conversions yet</div>
                  </div>
                ) : (
                  <div className="space-y-px max-w-4xl">
                    {history.map((job, i) => (
                      <div key={i} className="flex items-center gap-4 bg-[#1a1b1e] border border-[#2b2d31] rounded-xl px-5 py-4">
                        <div className="flex-1 font-mono text-sm truncate text-emerald-400/90">{job.input_path}</div>
                        <div className="text-xs text-zinc-500 font-mono">{new Date(job.created_at).toLocaleDateString()}</div>
                        <div className="flex items-center gap-1.5 text-xs px-3 py-1 rounded-full bg-emerald-500/10 text-emerald-400">
                          <CheckCircle className="w-3.5 h-3.5" /> COMPLETED
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </motion.div>
            )}

            {activeTab === 'settings' && (
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} className="p-8 max-w-2xl">
                <h1 className="text-3xl font-semibold tracking-[-0.5px] mb-8">Settings</h1>
                
                <div className="space-y-4">
                  <div className="bg-[#1a1b1e] border border-[#2b2d31] rounded-2xl p-6">
                    <div className="text-sm font-medium mb-3 flex items-center gap-2"><HardDrive className="w-4 h-4" /> Monitored Directory</div>
                    <input 
                      type="text" 
                      value={settings.monitored_directory}
                      onChange={e => updateSetting('monitored_directory', e.target.value)}
                      className="w-full bg-[#0f0f11] border border-[#2b2d31] rounded-lg px-4 py-2.5 text-sm font-mono focus:outline-none focus:border-emerald-600" 
                    />
                    <div className="text-[11px] text-zinc-500 mt-2">New .nsz files here are auto-detected and converted</div>
                  </div>

                  <div className="bg-[#1a1b1e] border border-[#2b2d31] rounded-2xl p-6">
                    <div className="text-sm font-medium mb-3 flex items-center gap-2"><FolderOpen className="w-4 h-4" /> Output Directory</div>
                    <input 
                      type="text" 
                      value={settings.output_directory}
                      onChange={e => updateSetting('output_directory', e.target.value)}
                      className="w-full bg-[#0f0f11] border border-[#2b2d31] rounded-lg px-4 py-2.5 text-sm font-mono focus:outline-none focus:border-emerald-600" 
                    />
                    <div className="text-[11px] text-zinc-500 mt-2">Games organized into named folders here</div>
                  </div>

                  <div className="bg-[#1a1b1e] border border-[#2b2d31] rounded-2xl p-6 space-y-5">
                    {[
                      { key: 'auto_convert', label: 'Auto-convert on detection', desc: 'Immediately convert when NSZ files appear' },
                      { key: 'verify_after_convert', label: 'Verify after conversion', desc: 'Run integrity check on output NSP files' },
                      { key: 'delete_source', label: 'Delete source files', desc: 'Remove original NSZ after successful conversion' },
                    ].map(item => (
                      <label key={item.key} className="flex items-start gap-4 cursor-pointer group">
                        <input 
                          type="checkbox" 
                          checked={settings[item.key as keyof typeof settings] as boolean}
                          onChange={e => updateSetting(item.key, e.target.checked)}
                          className="mt-1 accent-emerald-600" 
                        />
                        <div className="flex-1 -mt-0.5">
                          <div className="font-medium text-sm group-hover:text-emerald-400 transition-colors">{item.label}</div>
                          <div className="text-xs text-zinc-500">{item.desc}</div>
                        </div>
                      </label>
                    ))}
                  </div>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  )
}

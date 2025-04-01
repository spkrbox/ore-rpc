import { Settings } from '../types'

interface SettingsPageProps {
    userId: string
    settings: Settings
    onUserIdChange: (userId: string) => void
    onSettingsChange: (settings: Settings) => void
    onBack: () => void
}

export const SettingsPage = ({
    userId,
    settings,
    onUserIdChange,
    onSettingsChange,
    onBack,
}: SettingsPageProps) => {
    const handleSettingChange = (key: keyof Settings, value: boolean | string) => {
        onSettingsChange({ ...settings, [key]: value })
    }

    return (
        <div className='settings-page'>
            <div className='header'>
                <h1>Settings</h1>
                <button className='back-btn' onClick={onBack}>
                    ← Back
                </button>
            </div>

            <div className='settings-form'>
                <div className='setting-group user-id'>
                    <h2>User ID</h2>
                    <input
                        value={userId}
                        onChange={(e) => {
                            const newUserId = e.currentTarget.value
                            onUserIdChange(newUserId)
                            handleSettingChange('user_id', newUserId)
                        }}
                        placeholder='User ID'
                        style={{ width: '80%' }}
                    />
                </div>

                <div className='setting-group toggles'>
                    <h2>Display Options</h2>
                    <div className='toggle-grid'>
                        <ToggleOption
                            label='Discord RPC'
                            checked={settings.enabled}
                            onChange={(checked) => handleSettingChange('enabled', checked)}
                        />
                        <ToggleOption
                            label='Anime Title'
                            checked={settings.show_anime_title}
                            onChange={(checked) => handleSettingChange('show_anime_title', checked)}
                        />
                        <ToggleOption
                            label='Episode #'
                            checked={settings.show_episode_number}
                            onChange={(checked) =>
                                handleSettingChange('show_episode_number', checked)
                            }
                        />
                        <ToggleOption
                            label='Episode Title'
                            checked={settings.show_episode_title}
                            onChange={(checked) =>
                                handleSettingChange('show_episode_title', checked)
                            }
                        />
                        <ToggleOption
                            label='Timestamp'
                            checked={settings.show_timestamp}
                            onChange={(checked) => handleSettingChange('show_timestamp', checked)}
                        />
                        <ToggleOption
                            label='Progress'
                            checked={settings.show_progress}
                            onChange={(checked) => handleSettingChange('show_progress', checked)}
                        />
                    </div>
                </div>
            </div>
        </div>
    )
}

interface ToggleOptionProps {
    label: string
    checked: boolean
    onChange: (checked: boolean) => void
}

const ToggleOption = ({ label, checked, onChange }: ToggleOptionProps) => (
    <label className='toggle'>
        <input type='checkbox' checked={checked} onChange={(e) => onChange(e.target.checked)} />
        <span className='slider'></span>
        <span className='label'>{label}</span>
    </label>
)

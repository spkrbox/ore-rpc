import { WatchingData } from '../types'
import { formatTimeRemaining } from '../utils/formatters'

interface MainPageProps {
    watching: WatchingData | null
    onSettingsClick: () => void
}

export const MainPage = ({ watching, onSettingsClick }: MainPageProps) => (
    <div className='main-page'>
        <div className='header'>
            <h1>Ore Presence</h1>
            <button className='settings-btn' onClick={onSettingsClick}>
                ⚙️ Settings
            </button>
        </div>

        {watching ? (
            <div className='presence-card'>
                <div className='presence-header'>
                    <img src={watching.image} alt={watching.title} className='anime-image' />
                    <div className='presence-details'>
                        <h2>{watching.title}</h2>
                        {watching.episodeTitle ? (
                            <p>
                                Episode {watching.episodeNumber}: {watching.episodeTitle}
                            </p>
                        ) : (
                            <p>Episode {watching.episodeNumber}</p>
                        )}
                        <div className='progress-bar'>
                            <div
                                className='progress-fill'
                                style={{
                                    width: `${Math.min(
                                        100,
                                        (watching.progress / watching.duration) * 100
                                    )}%`,
                                }}
                            ></div>
                        </div>
                        <p className='time-remaining'>
                            {formatTimeRemaining(watching.startTimestamp, watching.endTimestamp)}
                        </p>
                    </div>
                </div>
            </div>
        ) : (
            <div className='no-watching'>
                <h2>Not currently watching anything</h2>
                <p>Start watching something to see your presence</p>
            </div>
        )}
    </div>
)

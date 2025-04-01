export interface Settings {
    user_id: string | null
    show_anime_title: boolean
    show_episode_title: boolean
    show_episode_number: boolean
    show_progress: boolean
    show_timestamp: boolean
    enabled: boolean
}

export interface WatchingData {
    id: string
    title: string
    animeId: string
    episodeId: string
    episodeNumber: number
    episodeTitle: string
    image: string
    progress: number
    duration: number
    timestamp: string
    updatedAt: string
    userId: string
    startTimestamp: number
    endTimestamp: number
}

export type Page = 'setup' | 'main' | 'settings'

export const formatTimeRemaining = (startTimestamp: number, endTimestamp: number): string => {
    try {
        if (!startTimestamp || !endTimestamp) {
            return 'Unknown'
        }

        const now = Date.now()
        const remaining = endTimestamp - now

        if (remaining <= 0) {
            return 'Finished'
        }

        const minutes = Math.floor(remaining / 60000)
        const seconds = Math.floor((remaining % 60000) / 1000)

        return `${minutes}:${seconds.toString().padStart(2, '0')} remaining`
    } catch (error) {
        console.error('Error formatting time:', error)
        return 'Unknown'
    }
}

export const formatProgress = (progress: number, duration: number): string => {
    const percent = (progress / duration) * 100
    return `${percent.toFixed(0)}%`
}

<script setup lang="ts">
import { useQuestsStore } from '@/stores/quests'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Button } from '@/components/ui/button'
import { AlertCircle } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const questsStore = useQuestsStore()

async function handleStop() {
  await questsStore.stop()
}
</script>

<template>
  <Card class="sticky top-20 border-border/50">
    <CardHeader>
      <CardTitle class="text-lg">{{ t('quest.active_progress') }}</CardTitle>
    </CardHeader>
    <CardContent>
      <div v-if="questsStore.activeQuestId" class="space-y-4">
        <div class="space-y-2">
          <div class="flex justify-between text-sm">
             <span class="text-muted-foreground truncate max-w-[150px]" :title="questsStore.activeQuestId">ID: {{ questsStore.activeQuestId.substring(0, 8) }}...</span>
             <span class="font-medium">{{ Math.round(questsStore.activeQuestProgress) }}%</span>
          </div>
          <Progress :model-value="questsStore.activeQuestProgress" class="h-3" />
        </div>
        
        <Button 
          variant="destructive" 
          class="w-full"
          @click="handleStop"
        >
          {{ t('home.stop_quest') }}
        </Button>
      </div>
      
      <div v-else class="text-center py-6 text-muted-foreground">
        {{ t('quest.no_active') }}
      </div>
      
      <div v-if="questsStore.questQueue.length > 0" class="mt-6 pt-4 border-t">
        <div class="flex justify-between items-center mb-2">
          <h4 class="font-semibold text-sm">{{ t('quest.up_next') }} ({{ questsStore.questQueue.length }})</h4>
          <Button 
            variant="ghost" 
            size="sm"
            class="h-6 px-2 text-destructive hover:text-destructive"
            @click="questsStore.clearQueue"
          >
            {{ t('general.clear') }}
          </Button>
        </div>
        
        <div class="space-y-2 max-h-[300px] overflow-y-auto pr-1">
          <div 
            v-for="(quest, index) in questsStore.questQueue" 
            :key="quest.id"
            class="bg-muted/50 p-2 rounded text-sm flex gap-2 items-center"
          >
            <span class="text-muted-foreground font-mono text-xs w-4">{{ index + 1 }}.</span>
            <div class="flex-1 overflow-hidden">
               <div class="truncate font-medium">{{ quest.config.messages.quest_name }}</div>
               <div class="text-xs text-muted-foreground truncate">{{ quest.config.messages.game_title }}</div>
            </div>
          </div>
        </div>
      </div>
  
      <div v-if="questsStore.error" class="mt-4 p-3 bg-red-500/10 border border-red-500/20 rounded text-sm text-red-500 flex items-start gap-2">
        <AlertCircle class="w-4 h-4 mt-0.5 shrink-0" />
        <span class="break-words">{{ questsStore.error }}</span>
      </div>
    </CardContent>
  </Card>
</template>


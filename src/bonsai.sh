#!/bin/bash

# --- Configuración ---
ESTIMATED_STEPS=450
BONSAI_DIR="~/Downloads/bonsais"

# --- Validaciones de Dependencias ---
if ! command -v cbonsai &>/dev/null || ! command -v bc &>/dev/null; then
    echo "❌ Error: 'cbonsai' o 'bc' no están instalados."
    exit 1
fi

# --- Función de Ayuda ---
show_help() {
    echo -e "\e[1;32m🌱 PomoBonsai - Tu Pomodoro gamificado en la terminal\e[0m\n"
    echo "Uso: $0 [OPCIONES]"
    echo ""
    echo "Opciones:"
    echo "  --timer <tiempo>    Inicia un Pomodoro. Acepta <minutos> o <min:seg>."
    echo "                      Ejemplos: $0 --timer 25  |  $0 --timer 25:30"
    echo "  --folder            Abre directamente la carpeta de tu jardín de bonsáis."
    echo "  -h, --help          Muestra este mensaje de ayuda."
    echo ""
    echo "Controles en ejecución:"
    echo "  [ P ]               Pausa / Reanuda el Pomodoro en cualquier momento."
    echo "  [ Ctrl + C ]        Interrumpe y mata el bonsái."
}

# --- Analizador de Argumentos CLI ---
ACTION="help"
TIMER_INPUT=""

while [[ "$#" -gt 0 ]]; do
    case $1 in
    --timer)
        TIMER_INPUT="$2"
        ACTION="timer"
        shift 2
        ;;
    --folder)
        ACTION="folder"
        shift
        ;;
    -h | --help)
        ACTION="help"
        shift
        ;;
    *)
        echo -e "❌ Argumento desconocido: \e[1;31m$1\e[0m\n"
        show_help
        exit 1
        ;;
    esac
done

# --- Ejecución: MODO FOLDER ---
if [ "$ACTION" == "folder" ]; then
    REAL_DIR="${BONSAI_DIR/#\~/$HOME}"
    mkdir -p "$REAL_DIR"
    echo -e "📂 Viajando a tu jardín de bonsáis en: \e[1;34m$REAL_DIR\e[0m"
    cd "$REAL_DIR" || exit 1
    # Abrimos una nueva terminal interactiva para que el usuario se quede en la carpeta
    exec "${SHELL:-bash}"
    exit 0
fi

# --- Ejecución: MODO HELP ---
if [ "$ACTION" == "help" ]; then
    show_help
    exit 0
fi

# --- Ejecución: MODO TIMER ---
if [ -z "$TIMER_INPUT" ]; then
    echo "❌ Error: Debes especificar un tiempo para el temporizador."
    echo "Ejemplo: $0 --timer 25"
    exit 1
fi

# Validar y extraer tiempo usando Regex
if [[ "$TIMER_INPUT" =~ ^([0-9]+):([0-5][0-9])$ ]]; then
    MINUTES=$((10#${BASH_REMATCH[1]}))
    SECONDS=$((10#${BASH_REMATCH[2]}))
elif [[ "$TIMER_INPUT" =~ ^[0-9]+$ ]]; then
    MINUTES=$TIMER_INPUT
    SECONDS=0
else
    echo "❌ Error: Formato de tiempo no válido."
    echo "Usa: <minutos> o <minutos:segundos> (ej. 25 o 25:30)"
    exit 1
fi

TOTAL_SECONDS=$((MINUTES * 60 + SECONDS))
DURATION_STR=$(printf "%02d:%02d" $MINUTES $SECONDS)

if [ "$TOTAL_SECONDS" -le 0 ]; then
    echo "❌ Error: El tiempo debe ser mayor a 0."
    exit 1
fi

DELAY=$(echo "scale=4; $TOTAL_SECONDS / $ESTIMATED_STEPS" | bc)
TREE_SEED=$RANDOM

# --- Variables de Estado ---
PAUSED=0
CBONSAI_PID=""
TIMER_PID=""
START_TIME=$(date +%s)
TOTAL_PAUSED=0
PAUSE_START=0
NEEDS_RESIZE=0

# --- Manejo de Interrupciones ---
cleanup() {
    stty sane
    tput rc
    tput cnorm
    [ -n "$TIMER_PID" ] && kill -9 $TIMER_PID 2>/dev/null
    [ -n "$CBONSAI_PID" ] && kill -9 $CBONSAI_PID 2>/dev/null
    echo -e "\n\n🛑 Pomodoro interrumpido. El bonsái ha muerto."
    exit 0
}
trap cleanup SIGINT SIGTERM

handle_resize() {
    NEEDS_RESIZE=1
}
trap handle_resize SIGWINCH

# --- Función Auxiliar Segura para Columnas ---
get_safe_col() {
    local cols=$(tput cols)
    local col_pos=$((cols - 15))
    if [ $col_pos -lt 0 ]; then col_pos=0; fi
    echo "$col_pos"
}

# --- Motor de Replantación Rápida ---
replant_bonsai() {
    local NOW=$(date +%s)
    local CURRENT_ELAPSED=$((NOW - START_TIME - TOTAL_PAUSED))
    if [ $PAUSED -eq 1 ]; then
        CURRENT_ELAPSED=$((PAUSE_START - START_TIME - TOTAL_PAUSED))
    fi

    local REMAINING=$((TOTAL_SECONDS - CURRENT_ELAPSED))
    if [ $REMAINING -le 0 ]; then REMAINING=1; fi

    [ -n "$CBONSAI_PID" ] && kill -9 $CBONSAI_PID 2>/dev/null

    clear
    local NEW_DELAY=$(echo "scale=4; $REMAINING / $ESTIMATED_STEPS" | bc)

    cbonsai -l -s "$TREE_SEED" -t "$NEW_DELAY" 2>/dev/null &
    CBONSAI_PID=$!
    disown $CBONSAI_PID 2>/dev/null

    if [ $PAUSED -eq 1 ]; then
        kill -STOP $CBONSAI_PID 2>/dev/null

        local m=$((REMAINING / 60))
        local s=$((REMAINING % 60))
        printf -v time_str "%02d:%02d" $m $s
        local safe_col=$(get_safe_col)

        tput sc
        tput cup 0 $safe_col
        echo -ne "\e[1;33m[ ⏳ $time_str ]\e[0m"
        tput cup 1 $safe_col
        echo -ne "\e[1;31m[ ⏸️ PAUSADO ]\e[0m"
        tput rc
    fi
}

# --- Función del Temporizador ---
show_timer() {
    local secs=$1
    tput civis
    while [ $secs -gt 0 ]; do
        local m=$((secs / 60))
        local s=$((secs % 60))
        printf -v time_str "%02d:%02d" $m $s

        local safe_col=$(get_safe_col)

        tput sc
        tput cup 0 $safe_col
        echo -ne "\e[1;33m[ ⏳ $time_str ]\e[0m"
        tput rc

        sleep 1
        ((secs--))
    done

    local safe_col=$(get_safe_col)
    tput sc
    tput cup 0 $safe_col
    echo -ne "\e[1;32m[ ✅ 00:00 ]\e[0m"
    tput rc
    tput cnorm
}

# --- Ejecución Principal ---
clear

show_timer $TOTAL_SECONDS &
TIMER_PID=$!
disown $TIMER_PID

sleep 0.1

cbonsai -l -s "$TREE_SEED" -t "$DELAY" 2>/dev/null &
CBONSAI_PID=$!
disown $CBONSAI_PID

stty -echo
tput civis

while kill -0 $TIMER_PID 2>/dev/null; do

    if [ $NEEDS_RESIZE -eq 1 ]; then
        NEEDS_RESIZE=0
        replant_bonsai
    fi

    read -t 0.5 -n 1 key

    if [[ "$key" == "p" || "$key" == "P" ]]; then
        local safe_col=$(get_safe_col)
        if [ $PAUSED -eq 0 ]; then
            PAUSE_START=$(date +%s)
            kill -STOP $TIMER_PID $CBONSAI_PID 2>/dev/null
            PAUSED=1

            tput sc
            tput cup 1 $safe_col
            echo -ne "\e[1;31m[ ⏸️ PAUSADO ]\e[0m"
            tput rc
        else
            local NOW=$(date +%s)
            TOTAL_PAUSED=$((TOTAL_PAUSED + NOW - PAUSE_START))
            kill -CONT $TIMER_PID $CBONSAI_PID 2>/dev/null
            PAUSED=0

            tput sc
            tput cup 1 $safe_col
            echo -ne "               "
            tput rc
        fi
    fi
done

# --- Restauración ---
stty sane
tput cnorm

wait $CBONSAI_PID 2>/dev/null

echo -e "\n\n🎉 ¡Pomodoro de $DURATION_STR completado! Has cultivado un hermoso bonsái."

# --- Guardar el Bonsái ---
echo ""
read -n 1 -p "❓ ¿Quieres guardar este bonsái en tu jardín? (s/n): " SAVE_CHOICE
echo ""

if [[ "$SAVE_CHOICE" =~ ^[sSyY]$ ]]; then
    read -p "🏷️  ¿Qué tarea has completado? (Déjalo en blanco si no quieres darle nombre): " RAW_NAME

    REAL_DIR="${BONSAI_DIR/#\~/$HOME}"
    mkdir -p "$REAL_DIR"

    ISO_DATE=$(date +"%Y-%m-%dT%H:%M:%S")

    if [ -n "$RAW_NAME" ]; then
        CLEAN_NAME=$(echo "$RAW_NAME" | tr ' ' '_')
        FILE_NAME="${ISO_DATE}-${DURATION_STR}-${CLEAN_NAME}.bs"
    else
        FILE_NAME="${ISO_DATE}-${DURATION_STR}.bs"
    fi

    FILE_PATH="$REAL_DIR/$FILE_NAME"

    cbonsai -p -s "$TREE_SEED" >"$FILE_PATH" 2>/dev/null

    if [ $? -eq 0 ]; then
        echo -e "✅ ¡Guardado! Creciendo feliz en: \e[1;32m$FILE_PATH\e[0m"
    else
        echo -e "❌ Hubo un error al guardar el bonsái."
    fi
else
    echo "🍂 El bonsái volverá a la naturaleza."
fi

// Debug: Check if Tauri API is available
console.log('Script loading...');

// Simple approach - try to use window.__TAURI__.invoke directly
console.log('Window.__TAURI__:', window.__TAURI__);

function getInvokeFunction() {
    if (window.__TAURI__ && window.__TAURI__.invoke) {
        console.log('Found invoke at window.__TAURI__.invoke');
        return window.__TAURI__.invoke;
    }
    
    if (window.__TAURI__ && window.__TAURI__.tauri && window.__TAURI__.tauri.invoke) {
        console.log('Found invoke at window.__TAURI__.tauri.invoke');
        return window.__TAURI__.tauri.invoke;
    }
    
    if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
        console.log('Found invoke at window.__TAURI__.core.invoke');
        return window.__TAURI__.core.invoke;
    }
    
    console.error('Could not find invoke function. Available:', window.__TAURI__);
    return null;
}

class UmaHelper {
    constructor() {
        console.log('UmaHelper constructor called');
        this.isCapturing = false;
        this.init();
    }

    init() {
        console.log('UmaHelper init called');
        this.setupEventListeners();
        this.setupResizableRectangle();
        this.updateStatus('Ready', 'success');
    }

    setupEventListeners() {
        console.log('Setting up event listeners...');
        
        // Capture button
        const captureBtn = document.getElementById('capture-btn');
        console.log('Capture button element:', captureBtn);
        
        if (captureBtn) {
            captureBtn.addEventListener('click', () => {
                console.log('CAPTURE BUTTON CLICKED!');
                this.startCapture();
            });
            console.log('Capture button listener added');
        }

        // Clear results
        const clearBtn = document.getElementById('clear-btn');
        if (clearBtn) {
            clearBtn.addEventListener('click', () => {
                console.log('CLEAR BUTTON CLICKED!');
                this.clearResults();
            });
            console.log('Clear button listener added');
        }
    }

    setupResizableRectangle() {
        const rectangle = document.getElementById('targeting-rectangle');
        if (!rectangle) return;

        let isResizing = false;
        let isDragging = false;
        let startX, startY, startWidth, startHeight, startTop, startLeft;

        // Handle drag to move
        rectangle.addEventListener('mousedown', (e) => {
            // Check if clicking on the bottom-right corner (resize handle)
            const rect = rectangle.getBoundingClientRect();
            const clickX = e.clientX;
            const clickY = e.clientY;
            const handleSize = 12;
            
            // Check if click is within bottom-right resize handle area
            if (clickX >= rect.right - handleSize && 
                clickY >= rect.bottom - handleSize) {
                isResizing = true;
                startX = e.clientX;
                startY = e.clientY;
                startWidth = rectangle.offsetWidth;
                startHeight = rectangle.offsetHeight;
            } else {
                // Regular drag to move
                isDragging = true;
                startX = e.clientX;
                startY = e.clientY;
                startTop = parseInt(getComputedStyle(rectangle).top);
                startLeft = parseInt(getComputedStyle(rectangle).left);
            }
            
            e.preventDefault();
        });

        // Global mouse move
        document.addEventListener('mousemove', (e) => {
            if (isDragging) {
                const deltaX = e.clientX - startX;
                const deltaY = e.clientY - startY;
                
                // Get capture column boundaries
                const captureColumn = document.querySelector('.capture-column');
                const columnRect = captureColumn.getBoundingClientRect();
                const columnWidth = captureColumn.offsetWidth;
                const columnHeight = captureColumn.offsetHeight;
                
                // Calculate boundaries within the capture column
                // Allow dragging below UI components by starting after the capture button area
                const minTop = 80; // Start below UI components (button + loading + status)
                const maxTop = columnHeight - rectangle.offsetHeight - 10; // Small margin at bottom
                const minLeft = 0;
                const maxLeft = columnWidth - rectangle.offsetWidth;
                
                const newTop = Math.max(minTop, Math.min(maxTop, startTop + deltaY));
                const newLeft = Math.max(minLeft, Math.min(maxLeft, startLeft + deltaX));
                
                rectangle.style.top = newTop + 'px';
                rectangle.style.left = newLeft + 'px';
                rectangle.style.right = 'auto';
                rectangle.style.bottom = 'auto';
            }
            
            if (isResizing) {
                const deltaX = e.clientX - startX;
                const deltaY = e.clientY - startY;
                
                // Get capture column boundaries for resizing
                const captureColumn = document.querySelector('.capture-column');
                const columnWidth = captureColumn.offsetWidth;
                const columnHeight = captureColumn.offsetHeight;
                const currentTop = parseInt(getComputedStyle(rectangle).top);
                const currentLeft = parseInt(getComputedStyle(rectangle).left);
                
                // Calculate maximum sizes based on current position
                const maxWidth = columnWidth - currentLeft;
                const maxHeight = columnHeight - currentTop - 10; // Small margin at bottom
                
                const newWidth = Math.max(100, Math.min(maxWidth, startWidth + deltaX));
                const newHeight = Math.max(50, Math.min(maxHeight, startHeight + deltaY));
                
                rectangle.style.width = newWidth + 'px';
                rectangle.style.height = newHeight + 'px';
                rectangle.style.right = 'auto';
                rectangle.style.bottom = 'auto';
            }
        });

        // Global mouse up
        document.addEventListener('mouseup', () => {
            isDragging = false;
            isResizing = false;
        });
    }

    async startCapture() {
        if (this.isCapturing) return;
        
        try {
            this.isCapturing = true;
            this.updateStatus('Capturing text...', 'processing');
            
            // Calculate the targeting rectangle area
            const targetingArea = await this.getTargetingArea();
            await this.captureArea(targetingArea);
            
        } catch (error) {
            console.error('Capture failed:', error);
            this.updateStatus(`Capture failed: ${error}`, 'error');
            this.isCapturing = false;
        }
    }

    async getTargetingArea() {
        try {
            // Get the targeting rectangle element
            const rectangle = document.getElementById('targeting-rectangle');
            if (!rectangle) {
                throw new Error('Targeting rectangle not found');
            }

            // Get rectangle position and size relative to capture column
            const rectRect = rectangle.getBoundingClientRect();
            const captureColumn = document.querySelector('.capture-column');
            const captureColumnRect = captureColumn.getBoundingClientRect();
            
            // Calculate screen coordinates relative to the capture column
            const screenX = window.screenX || window.screenLeft || 0;
            const screenY = window.screenY || window.screenTop || 0;
            
            // Account for window decorations (title bar, etc.)
            const windowDecorationHeight = (window.outerHeight - window.innerHeight);
            const titleBarHeight = windowDecorationHeight || 30;
            
            // Calculate the absolute screen position of the targeting rectangle
            // This ensures we capture only the area defined by the rectangle within the capture column
            const area = {
                x: Math.round(screenX + captureColumnRect.left + (rectRect.left - captureColumnRect.left)),
                y: Math.round(screenY + titleBarHeight + captureColumnRect.top + (rectRect.top - captureColumnRect.top)),
                width: Math.round(rectRect.width),
                height: Math.round(rectRect.height)
            };
            
            // Debug console output
            console.log('=== CAPTURE AREA ===');
            console.log('Rectangle rect:', rectRect);
            console.log('Capture column rect:', captureColumnRect);
            console.log('Screen position:', { screenX, screenY });
            console.log('Title bar offset:', titleBarHeight);
            console.log('Final screen area:', area);
            console.log('===================');
            
            return area;
            
        } catch (error) {
            console.error('Error calculating targeting area:', error);
            // Fallback to default rectangle size within capture column
            const captureColumn = document.querySelector('.capture-column');
            const captureColumnRect = captureColumn ? captureColumn.getBoundingClientRect() : null;
            
            if (captureColumnRect) {
                const screenX = window.screenX || window.screenLeft || 0;
                const screenY = window.screenY || window.screenTop || 0;
                const titleBarHeight = 30;
                
                return {
                    x: Math.round(screenX + captureColumnRect.left + 12),
                    y: Math.round(screenY + titleBarHeight + captureColumnRect.top + 12),
                    width: Math.round(captureColumnRect.width - 24),
                    height: Math.round(captureColumnRect.height - 80)
                };
            } else {
                // Final fallback if capture column not found
                return {
                    x: Math.round((window.screenX || 0) + 50),
                    y: Math.round((window.screenY || 0) + 80),
                    width: 200,
                    height: 150
                };
            }
        }
    }

    async showSelectionOverlay() {
        // Create fullscreen selection overlay
        const overlay = document.createElement('div');
        overlay.id = 'selection-overlay';
        overlay.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            width: 100vw;
            height: 100vh;
            background: rgba(0, 0, 0, 0.3);
            cursor: crosshair;
            z-index: 10000;
        `;

        const selectionRect = document.createElement('div');
        selectionRect.id = 'selection-rect';
        selectionRect.style.cssText = `
            position: absolute;
            border: 2px solid #007AFF;
            background: rgba(0, 122, 255, 0.1);
            pointer-events: none;
            display: none;
        `;

        overlay.appendChild(selectionRect);
        document.body.appendChild(overlay);

        let isSelecting = false;
        let startX, startY;

        const startSelection = (e) => {
            isSelecting = true;
            startX = e.clientX;
            startY = e.clientY;
            selectionRect.style.left = startX + 'px';
            selectionRect.style.top = startY + 'px';
            selectionRect.style.width = '0px';
            selectionRect.style.height = '0px';
            selectionRect.style.display = 'block';
        };

        const updateSelection = (e) => {
            if (!isSelecting) return;
            
            const currentX = e.clientX;
            const currentY = e.clientY;
            
            const left = Math.min(startX, currentX);
            const top = Math.min(startY, currentY);
            const width = Math.abs(currentX - startX);
            const height = Math.abs(currentY - startY);
            
            selectionRect.style.left = left + 'px';
            selectionRect.style.top = top + 'px';
            selectionRect.style.width = width + 'px';
            selectionRect.style.height = height + 'px';
        };

        const endSelection = async (e) => {
            if (!isSelecting) return;
            
            isSelecting = false;
            const currentX = e.clientX;
            const currentY = e.clientY;
            
            const area = {
                x: Math.min(startX, currentX),
                y: Math.min(startY, currentY),
                width: Math.abs(currentX - startX),
                height: Math.abs(currentY - startY)
            };

            document.body.removeChild(overlay);

            if (area.width > 10 && area.height > 10) {
                await this.captureArea(area);
            } else {
                this.updateStatus('Selection too small', 'error');
                this.isCapturing = false;
            }
        };

        const cancelSelection = (e) => {
            if (e.key === 'Escape') {
                document.body.removeChild(overlay);
                this.updateStatus('Capture cancelled', 'error');
                this.isCapturing = false;
            }
        };

        overlay.addEventListener('mousedown', startSelection);
        overlay.addEventListener('mousemove', updateSelection);
        overlay.addEventListener('mouseup', endSelection);
        document.addEventListener('keydown', cancelSelection);

        // Cleanup function
        overlay.cleanup = () => {
            document.removeEventListener('keydown', cancelSelection);
        };
    }

    async captureArea(area) {
        try {
            this.updateStatus('Processing image...', 'processing');
            this.showLoading();

            // Get invoke function dynamically
            const invoke = getInvokeFunction();
            if (!invoke) {
                throw new Error('Tauri invoke function not available');
            }

            console.log('Calling invoke with area:', area);
            console.log('Using invoke function:', invoke);
            
            // Perform OCR
            const ocrResult = await invoke('capture_screen_area', { area });
            
            this.hideLoading();
            
            if (ocrResult.text && ocrResult.text.trim()) {
                // Debug console output
                console.log('=== OCR RESULT ===');
                console.log('Raw text:', ocrResult.text);
                console.log('Trimmed text:', ocrResult.text.trim());
                console.log('Confidence:', ocrResult.confidence);
                console.log('Matched events:', ocrResult.matched_events);
                console.log('Text length:', ocrResult.text.length);
                console.log('==================');
                
                // Display matched events or recognized text
                if (ocrResult.matched_events && ocrResult.matched_events.length > 0) {
                    this.displayMatchedEvents(ocrResult.text, ocrResult.confidence, ocrResult.matched_events);
                    this.updateStatus(`Found ${ocrResult.matched_events.length} matching event(s)`, 'success');
                } else {
                    this.displayRecognizedText(ocrResult.text, ocrResult.confidence);
                    this.updateStatus(`Text recognized but no events matched (${ocrResult.confidence.toFixed(1)}% confidence)`, 'success');
                }
            } else {
                console.log('=== OCR RESULT ===');
                console.log('No text detected or empty result');
                console.log('Raw result:', ocrResult);
                console.log('==================');
                this.updateStatus('No text detected', 'error');
            }
            
        } catch (error) {
            console.error('OCR failed:', error);
            this.updateStatus(`OCR failed: ${error}`, 'error');
            this.hideLoading();
        } finally {
            this.isCapturing = false;
        }
    }

    displayMatchedEvents(extractedText, confidence, matchedEvents) {
        const eventSection = document.getElementById('event-result');
        const nameDiv = document.getElementById('event-name');
        const choicesDiv = document.getElementById('event-choices');
        
        nameDiv.textContent = `Matched Events (${matchedEvents.length} found)`;
        
        // Clear previous content
        choicesDiv.innerHTML = '';
        
        // Add extracted text section
        const textDiv = document.createElement('div');
        textDiv.style.cssText = 'padding: 8px; background: #f8f9fa; border-radius: 4px; margin-bottom: 12px; font-size: 11px;';
        textDiv.innerHTML = `
            <div style="font-weight: 600; margin-bottom: 4px; color: #2c3e50;">
                Extracted Text (${confidence.toFixed(1)}% OCR confidence):
            </div>
            <div style="background: white; padding: 6px; border-radius: 3px; font-family: monospace; border: 1px solid #dee2e6;">
                "${extractedText}"
            </div>
        `;
        choicesDiv.appendChild(textDiv);
        
        // Display each matched event
        matchedEvents.forEach((eventMatch, index) => {
            const eventDiv = document.createElement('div');
            eventDiv.style.cssText = `
                padding: 10px; 
                background: ${index === 0 ? '#e8f5e8' : '#f8f9fa'}; 
                border-radius: 4px; 
                margin-bottom: 8px; 
                border-left: 4px solid ${index === 0 ? '#28a745' : '#6c757d'};
            `;
            
            // Event header
            const headerDiv = document.createElement('div');
            headerDiv.style.cssText = 'font-weight: 600; margin-bottom: 6px; font-size: 12px; color: #2c3e50;';
            headerDiv.innerHTML = `
                ${index === 0 ? '🏆 ' : ''}${eventMatch.event.name}
                <span style="font-size: 10px; color: #6c757d; margin-left: 8px;">
                    ${(eventMatch.match_confidence * 100).toFixed(1)}% match (${eventMatch.match_type})
                </span>
            `;
            eventDiv.appendChild(headerDiv);
            
            // Character info
            if (eventMatch.event.character_name || eventMatch.event.relation_type) {
                const charDiv = document.createElement('div');
                charDiv.style.cssText = 'font-size: 10px; color: #666; margin-bottom: 8px;';
                charDiv.textContent = `${eventMatch.event.character_name} • ${eventMatch.event.relation_type}`;
                eventDiv.appendChild(charDiv);
            }
            
            // Choices
            if (eventMatch.event.choices && eventMatch.event.choices.length > 0) {
                const choicesContainer = document.createElement('div');
                choicesContainer.style.cssText = 'margin-top: 8px;';
                
                eventMatch.event.choices.forEach((choice, choiceIndex) => {
                    const choiceDiv = document.createElement('div');
                    choiceDiv.style.cssText = `
                        padding: 6px 8px; 
                        background: white; 
                        border-radius: 3px; 
                        margin-bottom: 4px; 
                        font-size: 10px;
                        border: 1px solid #e9ecef;
                    `;
                    
                    choiceDiv.innerHTML = `
                        <div style="font-weight: 600; margin-bottom: 2px;">
                            Choice ${choice.number}: ${choice.text}
                        </div>
                        <div style="color: #28a745; font-size: 9px;">
                            ${choice.outcome}
                        </div>
                    `;
                    choicesContainer.appendChild(choiceDiv);
                });
                
                eventDiv.appendChild(choicesContainer);
            }
            
            choicesDiv.appendChild(eventDiv);
        });
        
        // Show event result, hide no-result
        eventSection.classList.remove('hidden');
        document.getElementById('no-result').classList.add('hidden');
    }

    displayRecognizedText(text, confidence) {
        const eventSection = document.getElementById('event-result');
        const nameDiv = document.getElementById('event-name');
        const choicesDiv = document.getElementById('event-choices');
        
        nameDiv.textContent = 'No Events Matched';
        
        // Clear previous choices and show recognized text
        choicesDiv.innerHTML = `
            <div style="padding: 12px; background: #f8f9fa; border-radius: 4px; margin-bottom: 8px;">
                <div style="font-weight: 600; color: #2c3e50; margin-bottom: 6px; font-size: 12px;">
                    Extracted Text (${confidence.toFixed(1)}% confidence):
                </div>
                <div style="background: white; padding: 8px; border-radius: 3px; font-family: monospace; font-size: 11px; border: 1px solid #dee2e6;">
                    "${text}"
                </div>
                <div style="margin-top: 8px; font-size: 10px; color: #666; text-align: center;">
                    No matching events found in database
                </div>
            </div>
        `;
        
        // Show event result, hide no-result
        eventSection.classList.remove('hidden');
        document.getElementById('no-result').classList.add('hidden');
    }

    displayEventResult(event, recognizedText, confidence) {
        const eventSection = document.getElementById('event-result');
        const nameDiv = document.getElementById('event-name');
        const choicesDiv = document.getElementById('event-choices');
        
        nameDiv.textContent = event.event_name;
        
        // Clear previous choices
        choicesDiv.innerHTML = '';
        
        // Add recognized text section
        const textDiv = document.createElement('div');
        textDiv.style.cssText = 'padding: 8px; background: #f8f9fa; border-radius: 4px; margin-bottom: 8px; font-size: 11px;';
        textDiv.innerHTML = `
            <div style="font-weight: 600; margin-bottom: 4px;">Recognized: "${recognizedText}" (${confidence.toFixed(1)}%)</div>
        `;
        choicesDiv.appendChild(textDiv);
        
        // Add choices
        const choices = [
            { text: event.choice1_text, effects: event.choice1_effects },
            { text: event.choice2_text, effects: event.choice2_effects },
            { text: event.choice3_text, effects: event.choice3_effects }
        ];
        
        choices.forEach((choice, index) => {
            if (choice.text && choice.effects) {
                const choiceDiv = document.createElement('div');
                choiceDiv.className = 'choice';
                choiceDiv.innerHTML = `
                    <div class="choice-text">Choice ${index + 1}: ${choice.text}</div>
                    <div class="choice-effects">Effects: ${choice.effects}</div>
                `;
                choicesDiv.appendChild(choiceDiv);
            }
        });
        
        // Show event result, hide no-result
        eventSection.classList.remove('hidden');
        document.getElementById('no-result').classList.add('hidden');
    }

    displayNoEventFound(extractedText) {
        const eventSection = document.getElementById('event-result');
        const nameDiv = document.getElementById('event-name');
        const choicesDiv = document.getElementById('event-choices');
        
        nameDiv.textContent = 'Unknown Event';
        choicesDiv.innerHTML = `
            <div style="padding: 12px; text-align: center; color: #666; font-size: 11px;">
                <div style="margin-bottom: 8px;">Extracted text:</div>
                <div style="background: #f8f9fa; padding: 6px; border-radius: 3px; font-style: italic;">
                    "${extractedText}"
                </div>
                <div style="margin-top: 8px;">Event not found in database</div>
            </div>
        `;
        
        // Show event result, hide no-result
        eventSection.classList.remove('hidden');
        document.getElementById('no-result').classList.add('hidden');
    }

    clearResults() {
        document.getElementById('event-result').classList.add('hidden');
        document.getElementById('no-result').classList.remove('hidden');
        this.updateStatus('Results cleared', 'success');
    }

    showLoading() {
        document.getElementById('loading').classList.remove('hidden');
        document.getElementById('capture-btn').style.opacity = '0.5';
        document.getElementById('capture-btn').style.pointerEvents = 'none';
    }

    hideLoading() {
        document.getElementById('loading').classList.add('hidden');
        document.getElementById('capture-btn').style.opacity = '1';
        document.getElementById('capture-btn').style.pointerEvents = 'auto';
    }

    updateStatus(message, type = 'success') {
        const status = document.getElementById('status');
        status.textContent = message;
        status.className = `status ${type}`;
        
        // Auto-clear status after 3 seconds for non-error messages
        if (type !== 'error') {
            setTimeout(() => {
                if (status.textContent === message) {
                    status.textContent = 'Ready';
                    status.className = 'status';
                }
            }, 3000);
        }
    }
}

// Initialize the application when DOM is loaded
console.log('Adding DOMContentLoaded listener...');
document.addEventListener('DOMContentLoaded', () => {
    console.log('DOM Content Loaded! Initializing UmaHelper...');
    try {
        const helper = new UmaHelper();
        console.log('UmaHelper instance created:', helper);
    } catch (error) {
        console.error('Error creating UmaHelper:', error);
    }
});

// Also try immediate initialization in case DOM is already loaded
if (document.readyState === 'loading') {
    console.log('Document still loading, waiting for DOMContentLoaded...');
} else {
    console.log('Document already loaded, initializing immediately...');
    try {
        const helper = new UmaHelper();
        console.log('UmaHelper instance created immediately:', helper);
    } catch (error) {
        console.error('Error creating UmaHelper immediately:', error);
    }
}

// Handle window focus/blur for better UX
window.addEventListener('blur', () => {
    document.body.style.opacity = '0.8';
});

window.addEventListener('focus', () => {
    document.body.style.opacity = '1';
});
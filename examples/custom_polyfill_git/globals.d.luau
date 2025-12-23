-- this file relies on only thoughts and prayers to work thanks to innefficient parsing & memory leaks from lsp 🙏
-- modify this at your risk
-- jit types coming soon (if luau-lsp fixes the typedef files parsing or i find a hack to make it work)

declare ffi: any
declare jit: any

type userdata = typeof(newproxy())
type Variant = any
type DistanceModel = 'none'|'inverse'|'inverseclamped'|'linear'|'linearclamped'|'exponent'|'exponentclamped'
type EffectType = 'chorus'|'compressor'|'distortion'|'echo'|'equalizer'|'flanger'|'reverb'|'ringmodulator'
type EffectWaveform = 'sawtooth'|'sine'|'square'|'triangle'
type FilterType = 'lowpass'|'highpass'|'bandpass'
type SourceType = 'static'|'stream'|'queue'
type TimeUnit = 'seconds'|'samples'
type CompressedDataFormat = 'lz4'|'zlib'|'gzip'|'deflate'
type ContainerType = 'data'|'string'
type EncodeFormat = 'base64'|'hex'
type HashFunction = 'md5'|'sha1'|'sha224'|'sha256'|'sha384'|'sha512'
type Event = 'focus'|'joystickpressed'|'joystickreleased'|'keypressed'|'keyreleased'|'mousepressed'|'mousereleased'|'quit'|'resize'|'visible'|'mousefocus'|'threaderror'|'joystickadded'|'joystickremoved'|'joystickaxis'|'joystickhat'|'gamepadpressed'|'gamepadreleased'|'gamepadaxis'|'textinput'|'mousemoved'|'lowmemory'|'textedited'|'wheelmoved'|'touchpressed'|'touchreleased'|'touchmoved'|'directorydropped'|'filedropped'|'jp'|'jr'|'kp'|'kr'|'mp'|'mr'|'q'|'f'
type BufferMode = 'none'|'line'|'full'
type FileDecoder = 'file'|'base64'
type FileMode = 'r'|'w'|'a'|'c'
type FileType = 'file'|'directory'|'symlink'|'other'
type HintingMode = 'normal'|'light'|'mono'|'none'
type AlignMode = 'center'|'left'|'right'|'justify'
type ArcType = 'pie'|'open'|'closed'
type AreaSpreadDistribution = 'uniform'|'normal'|'ellipse'|'borderellipse'|'borderrectangle'|'none'
type BlendAlphaMode = 'alphamultiply'|'premultiplied'
type BlendMode = 'alpha'|'replace'|'screen'|'add'|'subtract'|'multiply'|'lighten'|'darken'|'additive'|'subtractive'|'multiplicative'|'premultiplied'
type CompareMode = 'equal'|'notequal'|'less'|'lequal'|'gequal'|'greater'|'never'|'always'
type CullMode = 'back'|'front'|'none'
type DrawMode = 'fill'|'line'
type FilterMode = 'linear'|'nearest'
type GraphicsFeature = 'clampzero'|'lighten'|'multicanvasformats'|'glsl3'|'instancing'|'fullnpot'|'pixelshaderhighp'|'shaderderivatives'
type GraphicsLimit = 'pointsize'|'texturesize'|'multicanvas'|'canvasmsaa'|'texturelayers'|'volumetexturesize'|'cubetexturesize'|'anisotropy'
type IndexDataType = 'uint16'|'uint32'
type LineJoin = 'miter'|'none'|'bevel'
type LineStyle = 'rough'|'smooth'
type MeshDrawMode = 'fan'|'strip'|'triangles'|'points'
type MipmapMode = 'none'|'auto'|'manual'
type ParticleInsertMode = 'top'|'bottom'|'random'
type SpriteBatchUsage = 'dynamic'|'static'|'stream'
type StackType = 'transform'|'all'
type StencilAction = 'replace'|'increment'|'decrement'|'incrementwrap'|'decrementwrap'|'invert'
type TextureType = '2d'|'array'|'cube'|'volume'
type VertexAttributeStep = 'pervertex'|'perinstance'
type VertexWinding = 'cw'|'ccw'
type WrapMode = 'clamp'|'repeat'|'mirroredrepeat'|'clampzero'
type CompressedImageFormat = 'DXT1'|'DXT3'|'DXT5'|'BC4'|'BC4s'|'BC5'|'BC5s'|'BC6h'|'BC6hs'|'BC7'|'ETC1'|'ETC2rgb'|'ETC2rgba'|'ETC2rgba1'|'EACr'|'EACrs'|'EACrg'|'EACrgs'|'PVR1rgb2'|'PVR1rgb4'|'PVR1rgba2'|'PVR1rgba4'|'ASTC4x4'|'ASTC5x4'|'ASTC5x5'|'ASTC6x5'|'ASTC6x6'|'ASTC8x5'|'ASTC8x6'|'ASTC8x8'|'ASTC10x5'|'ASTC10x6'|'ASTC10x8'|'ASTC10x10'|'ASTC12x10'|'ASTC12x12'
type ImageFormat = 'tga'|'png'|'jpg'|'bmp'
type PixelFormat = 'unknown'|'normal'|'hdr'|'r8'|'rg8'|'rgba8'|'srgba8'|'r16'|'rg16'|'rgba16'|'r16f'|'rg16f'|'rgba16f'|'r32f'|'rg32f'|'rgba32f'|'la8'|'rgba4'|'rgb5a1'|'rgb565'|'rgb10a2'|'rg11b10f'|'stencil8'|'depth16'|'depth24'|'depth32f'|'depth24stencil8'|'depth32fstencil8'|'DXT1'|'DXT3'|'DXT5'|'BC4'|'BC4s'|'BC5'|'BC5s'|'BC6h'|'BC6hs'|'BC7'|'ETC1'|'ETC2rgb'|'ETC2rgba'|'ETC2rgba1'|'EACr'|'EACrs'|'EACrg'|'EACrgs'|'PVR1rgb2'|'PVR1rgb4'|'PVR1rgba2'|'PVR1rgba4'|'ASTC4x4'|'ASTC5x4'|'ASTC5x5'|'ASTC6x5'|'ASTC6x6'|'ASTC8x5'|'ASTC8x6'|'ASTC8x8'|'ASTC10x5'|'ASTC10x6'|'ASTC10x8'|'ASTC10x10'|'ASTC12x10'|'ASTC12x12'
type GamepadAxis = 'leftx'|'lefty'|'rightx'|'righty'|'triggerleft'|'triggerright'
type GamepadButton = 'a'|'b'|'x'|'y'|'back'|'guide'|'start'|'leftstick'|'rightstick'|'leftshoulder'|'rightshoulder'|'dpup'|'dpdown'|'dpleft'|'dpright'
type JoystickHat = 'c'|'d'|'l'|'ld'|'lu'|'r'|'rd'|'ru'|'u'
type JoystickInputType = 'axis'|'button'|'hat'
type KeyConstant = 'a'|'b'|'c'|'d'|'e'|'f'|'g'|'h'|'i'|'j'|'k'|'l'|'m'|'n'|'o'|'p'|'q'|'r'|'s'|'t'|'u'|'v'|'w'|'x'|'y'|'z'|'0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'space'|'!'|'\"'|'#'|'$'|'&'|'\''|'('|')'|'*'|'+'|','|'-'|'.'|'/'|':'|';'|'<'|'='|'>'|'?'|'@'|'['|'\\'|']'|'^'|'_'|'`'|'kp0'|'kp1'|'kp2'|'kp3'|'kp4'|'kp5'|'kp6'|'kp7'|'kp8'|'kp9'|'kp.'|'kp/'|'kp*'|'kp-'|'kp+'|'kpenter'|'kp='|'up'|'down'|'right'|'left'|'home'|'end'|'pageup'|'pagedown'|'insert'|'backspace'|'tab'|'clear'|'return'|'delete'|'f1'|'f2'|'f3'|'f4'|'f5'|'f6'|'f7'|'f8'|'f9'|'f10'|'f11'|'f12'|'f13'|'f14'|'f15'|'numlock'|'capslock'|'scrollock'|'rshift'|'lshift'|'rctrl'|'lctrl'|'ralt'|'lalt'|'rmeta'|'lmeta'|'lsuper'|'rsuper'|'mode'|'compose'|'pause'|'escape'|'help'|'print'|'sysreq'|'break'|'menu'|'power'|'euro'|'undo'|'www'|'mail'|'calculator'|'appsearch'|'apphome'|'appback'|'appforward'|'apprefresh'|'appbookmarks'
type Scancode = 'a'|'b'|'c'|'d'|'e'|'f'|'g'|'h'|'i'|'j'|'k'|'l'|'m'|'n'|'o'|'p'|'q'|'r'|'s'|'t'|'u'|'v'|'w'|'x'|'y'|'z'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'0'|'return'|'escape'|'backspace'|'tab'|'space'|'-'|'='|'['|']'|'\\'|'nonus#'|';'|'\''|'`'|','|'.'|'/'|'capslock'|'f1'|'f2'|'f3'|'f4'|'f5'|'f6'|'f7'|'f8'|'f9'|'f10'|'f11'|'f12'|'f13'|'f14'|'f15'|'f16'|'f17'|'f18'|'f19'|'f20'|'f21'|'f22'|'f23'|'f24'|'lctrl'|'lshift'|'lalt'|'lgui'|'rctrl'|'rshift'|'ralt'|'rgui'|'printscreen'|'scrolllock'|'pause'|'insert'|'home'|'numlock'|'pageup'|'delete'|'end'|'pagedown'|'right'|'left'|'down'|'up'|'nonusbackslash'|'application'|'execute'|'help'|'menu'|'select'|'stop'|'again'|'undo'|'cut'|'copy'|'paste'|'find'|'kp/'|'kp*'|'kp-'|'kp+'|'kp='|'kpenter'|'kp1'|'kp2'|'kp3'|'kp4'|'kp5'|'kp6'|'kp7'|'kp8'|'kp9'|'kp0'|'kp.'|'international1'|'international2'|'international3'|'international4'|'international5'|'international6'|'international7'|'international8'|'international9'|'lang1'|'lang2'|'lang3'|'lang4'|'lang5'|'mute'|'volumeup'|'volumedown'|'audionext'|'audioprev'|'audiostop'|'audioplay'|'audiomute'|'mediaselect'|'www'|'mail'|'calculator'|'computer'|'acsearch'|'achome'|'acback'|'acforward'|'acstop'|'acrefresh'|'acbookmarks'|'power'|'brightnessdown'|'brightnessup'|'displayswitch'|'kbdillumtoggle'|'kbdillumdown'|'kbdillumup'|'eject'|'sleep'|'alterase'|'sysreq'|'cancel'|'clear'|'prior'|'return2'|'separator'|'out'|'oper'|'clearagain'|'crsel'|'exsel'|'kp00'|'kp000'|'thsousandsseparator'|'decimalseparator'|'currencyunit'|'currencysubunit'|'app1'|'app2'|'unknown'
type MatrixLayout = 'row'|'column'
type CursorType = 'image'|'arrow'|'ibeam'|'wait'|'waitarrow'|'crosshair'|'sizenwse'|'sizenesw'|'sizewe'|'sizens'|'sizeall'|'no'|'hand'
type BodyType = 'static'|'dynamic'|'kinematic'
type JointType = 'distance'|'friction'|'gear'|'mouse'|'prismatic'|'pulley'|'revolute'|'rope'|'weld'
type ShapeType = 'circle'|'polygon'|'edge'|'chain'
type PowerState = 'unknown'|'battery'|'nobattery'|'charging'|'charged'
type DisplayOrientation = 'unknown'|'landscape'|'landscapeflipped'|'portrait'|'portraitflipped'
type FullscreenType = 'desktop'|'exclusive'|'normal'
type MessageBoxType = 'info'|'warning'|'error'

declare class Object
	function release(self): (boolean)
	function type(self): (string)
	function typeOf(self,name:string): (boolean)
end

declare class Data extends Object
	function clone(self): (Data)
	function getFFIPointer(self): (unknown)
	function getPointer(self): (unknown)
	function getSize(self): (number)
	function getString(self): (string)
end

declare class RecordingDevice extends Object
	function getBitDepth(self): (number)
	function getChannelCount(self): (number)
	function getData(self): (SoundData)
	function getName(self): (string)
	function getSampleCount(self): (number)
	function getSampleRate(self): (number)
	function isRecording(self): (boolean)
	function start(self,samplecount:number,samplerate:number,bitdepth:number,channels:number): (boolean)
	function stop(self): (SoundData)
end

declare class Source extends Object
	function clone(self): (Source)
	function getActiveEffects(self): ({string})
	function getAirAbsorption(self): (number)
	function getAttenuationDistances(self): (number,number)
	function getChannelCount(self): (number)
	function getCone(self): (number,number,number)
	function getDirection(self): (number,number,number)
	function getDuration(self,unit:TimeUnit): (number)
	function getEffect(self,name:string,filtersettings:{[any]: any}): ({volume:number,highgain:number,lowgain:number,})
	function getFilter(self): ({type:FilterType,volume:number,highgain:number,lowgain:number,})
	function getFreeBufferCount(self): (number)
	function getPitch(self): (number)
	function getPosition(self): (number,number,number)
	function getRolloff(self): (number)
	function getType(self): (SourceType)
	function getVelocity(self): (number,number,number)
	function getVolume(self): (number)
	function getVolumeLimits(self): (number,number)
	function isLooping(self): (boolean)
	function isPlaying(self): (boolean)
	function isRelative(self): (boolean)
	function pause(self): ()
	function play(self): (boolean)
	function queue(self,sounddata:SoundData): (boolean)
	function seek(self,offset:number,unit:TimeUnit): ()
	function setAirAbsorption(self,amount:number): ()
	function setAttenuationDistances(self,ref:number,max:number): ()
	function setCone(self,innerAngle:number,outerAngle:number,outerVolume:number): ()
	function setDirection(self,x:number,y:number,z:number): ()
	function setEffect(self,name:string,enable:boolean): (boolean)
	function setEffect(self,name:string,filtersettings:{type:FilterType,volume:number,highgain:number,lowgain:number,}): (boolean)
	function setFilter(self,settings:{type:FilterType,volume:number,highgain:number,lowgain:number,}): (boolean)
	function setFilter(self): ()
	function setLooping(self,loop:boolean): ()
	function setPitch(self,pitch:number): ()
	function setPosition(self,x:number,y:number,z:number): ()
	function setRelative(self,enable:boolean): ()
	function setRolloff(self,rolloff:number): ()
	function setVelocity(self,x:number,y:number,z:number): ()
	function setVolume(self,volume:number): ()
	function setVolumeLimits(self,min:number,max:number): ()
	function stop(self): ()
	function tell(self,unit:TimeUnit): (number)
end

declare class ByteData extends Object
end

declare class CompressedData extends Data
	function getFormat(self): (CompressedDataFormat)
end

declare class File extends Object
	function close(self): (boolean)
	function flush(self): (boolean,string)
	function getBuffer(self): (BufferMode,number)
	function getFilename(self): (string)
	function getMode(self): (FileMode)
	function getSize(self): (number)
	function isEOF(self): (boolean)
	function isOpen(self): (boolean)
	function lines(self): ((...any)->(...any))
	function open(self,mode:FileMode): (boolean,string)
	function read(self,bytes:number): (string,number)
	function read(self,container:ContainerType,bytes:number): (FileData|string,number)
	function seek(self,pos:number): (boolean)
	function setBuffer(self,mode:BufferMode,size:number): (boolean,string)
	function tell(self): (number)
	function write(self,data:string,size:number): (boolean,string)
	function write(self,data:Data,size:number): (boolean,string)
end

declare class DroppedFile extends File
end

declare class FileData extends Data
	function getExtension(self): (string)
	function getFilename(self): (string)
end

declare class GlyphData extends Data
	function getAdvance(self): (number)
	function getBearing(self): (number,number)
	function getBoundingBox(self): (number,number,number,number)
	function getDimensions(self): (number,number)
	function getFormat(self): (PixelFormat)
	function getGlyph(self): (number)
	function getGlyphString(self): (string)
	function getHeight(self): (number)
	function getWidth(self): (number)
end

declare class Rasterizer extends Object
	function getAdvance(self): (number)
	function getAscent(self): (number)
	function getDescent(self): (number)
	function getGlyphCount(self): (number)
	function getGlyphData(self,glyph:string): (GlyphData)
	function getGlyphData(self,glyphNumber:number): (GlyphData)
	function getHeight(self): (number)
	function getLineHeight(self): (number)
	function hasGlyphs(self,glyph1:string|number,glyph2:string|number,...:string|number): (boolean)
end

declare class Drawable extends Object
end

declare class Texture extends Drawable
	function getDPIScale(self): (number)
	function getDepth(self): (number)
	function getDepthSampleMode(self): (CompareMode)
	function getDimensions(self): (number,number)
	function getFilter(self): (FilterMode,FilterMode,number)
	function getFormat(self): (PixelFormat)
	function getHeight(self): (number)
	function getLayerCount(self): (number)
	function getMipmapCount(self): (number)
	function getMipmapFilter(self): (FilterMode,number)
	function getPixelDimensions(self): (number,number)
	function getPixelHeight(self): (number)
	function getPixelWidth(self): (number)
	function getTextureType(self): (TextureType)
	function getWidth(self): (number)
	function getWrap(self): (WrapMode,WrapMode,WrapMode)
	function isReadable(self): (boolean)
	function setDepthSampleMode(self,compare:CompareMode): ()
	function setFilter(self,min:FilterMode,mag:FilterMode,anisotropy:number): ()
	function setMipmapFilter(self,filtermode:FilterMode,sharpness:number): ()
	function setMipmapFilter(self): ()
	function setWrap(self,horiz:WrapMode,vert:WrapMode,depth:WrapMode): ()
end

declare class Canvas extends Texture
	function generateMipmaps(self): ()
	function getMSAA(self): (number)
	function getMipmapMode(self): (MipmapMode)
	function newImageData(self): (ImageData)
	function newImageData(self,slice:number,mipmap:number,x:number,y:number,width:number,height:number): (ImageData)
	function renderTo(self,func:(...any)->(...any),...:any): ()
end

declare class Font extends Object
	function getAscent(self): (number)
	function getBaseline(self): (number)
	function getDPIScale(self): (number)
	function getDescent(self): (number)
	function getFilter(self): (FilterMode,FilterMode,number)
	function getHeight(self): (number)
	function getKerning(self,leftchar:string,rightchar:string): (number)
	function getKerning(self,leftglyph:number,rightglyph:number): (number)
	function getLineHeight(self): (number)
	function getWidth(self,text:string): (number)
	function getWrap(self,text:string,wraplimit:number): (number,{string})
	function hasGlyphs(self,text:string): (boolean)
	function hasGlyphs(self,character1:string,character2:string): (boolean)
	function hasGlyphs(self,codepoint1:number,codepoint2:number): (boolean)
	function setFallbacks(self,fallbackfont1:Font,...:Font): ()
	function setFilter(self,min:FilterMode,mag:FilterMode,anisotropy:number): ()
	function setLineHeight(self,height:number): ()
end

declare class Image extends Texture
	function isCompressed(self): (boolean)
	function isFormatLinear(self): (boolean)
	function replacePixels(self,data:ImageData,slice:number,mipmap:number,x:number,y:number,reloadmipmaps:boolean): ()
end

declare class Mesh extends Drawable
	function attachAttribute(self,name:string,mesh:Mesh): ()
	function attachAttribute(self,name:string,mesh:Mesh,step:VertexAttributeStep,attachname:string): ()
	function detachAttribute(self,name:string): (boolean)
	function flush(self): ()
	function getDrawMode(self): (MeshDrawMode)
	function getDrawRange(self): (number,number)
	function getTexture(self): (Texture)
	function getVertex(self,index:number): (number,number)
	function getVertex(self,index:number): (number,number,number,number,number,number,number,number)
	function getVertexAttribute(self,vertexindex:number,attributeindex:number): (number,number,number)
	function getVertexCount(self): (number)
	function getVertexFormat(self): ({attribute:{[any]: any},})
	function getVertexMap(self): ({number})
	function isAttributeEnabled(self,name:string): (boolean)
	function setAttributeEnabled(self,name:string,enable:boolean): ()
	function setDrawMode(self,mode:MeshDrawMode): ()
	function setDrawRange(self,start:number,count:number): ()
	function setDrawRange(self): ()
	function setTexture(self,texture:Texture): ()
	function setTexture(self): ()
	function setVertex(self,index:number,attributecomponent:number,...:number): ()
	function setVertex(self,index:number,vertex:{attributecomponent:number,}): ()
	function setVertex(self,index:number,x:number,y:number,u:number,v:number,r:number,g:number,b:number,a:number): ()
	function setVertex(self,index:number,vertex:{number?}): ()
	function setVertexAttribute(self,vertexindex:number,attributeindex:number,value1:number,value2:number,...:number): ()
	function setVertexMap(self,map:{[any]: any}): ()
	function setVertexMap(self,vi1:number,vi2:number,vi3:number): ()
	function setVertexMap(self,data:Data,datatype:IndexDataType): ()
	function setVertices(self,vertices:{attributecomponent:number,},startvertex:number,count:number): ()
	function setVertices(self,data:Data,startvertex:number): ()
	function setVertices(self,vertices:{number?}): ()
end

declare class ParticleSystem extends Drawable
	function clone(self): (ParticleSystem)
	function emit(self,numparticles:number): ()
	function getBufferSize(self): (number)
	function getColors(self): (number,number,number,number,number,number,number,number,number,number,number,number)
	function getCount(self): (number)
	function getDirection(self): (number)
	function getEmissionArea(self): (AreaSpreadDistribution,number,number,number,boolean)
	function getEmissionRate(self): (number)
	function getEmitterLifetime(self): (number)
	function getInsertMode(self): (ParticleInsertMode)
	function getLinearAcceleration(self): (number,number,number,number)
	function getLinearDamping(self): (number,number)
	function getOffset(self): (number,number)
	function getParticleLifetime(self): (number,number)
	function getPosition(self): (number,number)
	function getQuads(self): ({Quad})
	function getRadialAcceleration(self): (number,number)
	function getRotation(self): (number,number)
	function getSizeVariation(self): (number)
	function getSizes(self): (number,number,number)
	function getSpeed(self): (number,number)
	function getSpin(self): (number,number,number)
	function getSpinVariation(self): (number)
	function getSpread(self): (number)
	function getTangentialAcceleration(self): (number,number)
	function getTexture(self): (Texture)
	function hasRelativeRotation(self): (boolean)
	function isActive(self): (boolean)
	function isPaused(self): (boolean)
	function isStopped(self): (boolean)
	function moveTo(self,x:number,y:number): ()
	function pause(self): ()
	function reset(self): ()
	function setBufferSize(self,size:number): ()
	function setColors(self,r1:number,g1:number,b1:number,a1:number,...:number): ()
	function setColors(self,rgba1:{number},...:{number}): ()
	function setDirection(self,direction:number): ()
	function setEmissionArea(self,distribution:AreaSpreadDistribution,dx:number,dy:number,angle:number,directionRelativeToCenter:boolean): ()
	function setEmissionRate(self,rate:number): ()
	function setEmitterLifetime(self,life:number): ()
	function setInsertMode(self,mode:ParticleInsertMode): ()
	function setLinearAcceleration(self,xmin:number,ymin:number,xmax:number,ymax:number): ()
	function setLinearDamping(self,min:number,max:number): ()
	function setOffset(self,x:number,y:number): ()
	function setParticleLifetime(self,min:number,max:number): ()
	function setPosition(self,x:number,y:number): ()
	function setQuads(self,quad1:Quad,...:Quad): ()
	function setQuads(self,quads:{Quad}): ()
	function setRadialAcceleration(self,min:number,max:number): ()
	function setRelativeRotation(self,enable:boolean): ()
	function setRotation(self,min:number,max:number): ()
	function setSizeVariation(self,variation:number): ()
	function setSizes(self,size1:number,size2:number,size8:number): ()
	function setSpeed(self,min:number,max:number): ()
	function setSpin(self,min:number,max:number): ()
	function setSpinVariation(self,variation:number): ()
	function setSpread(self,spread:number): ()
	function setTangentialAcceleration(self,min:number,max:number): ()
	function setTexture(self,texture:Texture): ()
	function start(self): ()
	function stop(self): ()
	function update(self,dt:number): ()
end

declare class Quad extends Object
	function getTextureDimensions(self): (number,number)
	function getViewport(self): (number,number,number,number)
	function setViewport(self,x:number,y:number,w:number,h:number,sw:number,sh:number): ()
end

declare class Shader extends Object
	function getWarnings(self): (string)
	function hasUniform(self,name:string): (boolean)
	function send(self,name:string,number:number,...:number): ()
	function send(self,name:string,vector:{[any]: any},...:{[any]: any}): ()
	function send(self,name:string,matrix:{[any]: any},...:{[any]: any}): ()
	function send(self,name:string,texture:Texture): ()
	function send(self,name:string,boolean:boolean,...:boolean): ()
	function send(self,name:string,matrixlayout:MatrixLayout,matrix:{[any]: any},...:{[any]: any}): ()
	function send(self,name:string,data:Data,offset:number,size:number): ()
	function send(self,name:string,data:Data,matrixlayout:MatrixLayout,offset:number,size:number): ()
	function send(self,name:string,matrixlayout:MatrixLayout,data:Data,offset:number,size:number): ()
	function sendColor(self,name:string,color:{number},...:{number}): ()
end

declare class SpriteBatch extends Drawable
	function add(self,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): (number)
	function add(self,quad:Quad,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): (number)
	function addLayer(self,layerindex:number,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): (number)
	function addLayer(self,layerindex:number,quad:Quad,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): (number)
	function addLayer(self,layerindex:number,transform:Transform): (number)
	function addLayer(self,layerindex:number,quad:Quad,transform:Transform): (number)
	function attachAttribute(self,name:string,mesh:Mesh): ()
	function clear(self): ()
	function flush(self): ()
	function getBufferSize(self): (number)
	function getColor(self): (number,number,number,number)
	function getCount(self): (number)
	function getTexture(self): (Texture)
	function set(self,spriteindex:number,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): ()
	function set(self,spriteindex:number,quad:Quad,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): ()
	function setColor(self,r:number,g:number,b:number,a:number): ()
	function setColor(self): ()
	function setDrawRange(self,start:number,count:number): ()
	function setDrawRange(self): ()
	function setLayer(self,spriteindex:number,layerindex:number,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): ()
	function setLayer(self,spriteindex:number,layerindex:number,quad:Quad,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): ()
	function setLayer(self,spriteindex:number,layerindex:number,transform:Transform): ()
	function setLayer(self,spriteindex:number,layerindex:number,quad:Quad,transform:Transform): ()
	function setTexture(self,texture:Texture): ()
end

declare class Text extends Drawable
	function add(self,textstring:string,x:number,y:number,angle:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): (number)
	function add(self,coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string,},x:number,y:number,angle:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): (number)
	function addf(self,textstring:string,wraplimit:number,align:AlignMode,x:number,y:number,angle:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): (number)
	function addf(self,coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string,},wraplimit:number,align:AlignMode,x:number,y:number,angle:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): (number)
	function clear(self): ()
	function getDimensions(self): (number,number)
	function getDimensions(self,index:number): (number,number)
	function getFont(self): (Font)
	function getHeight(self): (number)
	function getHeight(self,index:number): (number)
	function getWidth(self): (number)
	function getWidth(self,index:number): (number)
	function set(self,textstring:string): ()
	function set(self,coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string,}): ()
	function setFont(self,font:Font): ()
	function setf(self,textstring:string,wraplimit:number,align:AlignMode): ()
	function setf(self,coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string,},wraplimit:number,align:AlignMode): ()
end

declare class Video extends Drawable
	function getDimensions(self): (number,number)
	function getFilter(self): (FilterMode,FilterMode,number)
	function getHeight(self): (number)
	function getSource(self): (Source)
	function getStream(self): (VideoStream)
	function getWidth(self): (number)
	function isPlaying(self): (boolean)
	function pause(self): ()
	function play(self): ()
	function rewind(self): ()
	function seek(self,offset:number): ()
	function setFilter(self,min:FilterMode,mag:FilterMode,anisotropy:number): ()
	function setSource(self,source:Source): ()
	function tell(self): (number)
end

declare class CompressedImageData extends Data
	function getDimensions(self): (number,number)
	function getDimensions(self,level:number): (number,number)
	function getFormat(self): (CompressedImageFormat)
	function getHeight(self): (number)
	function getHeight(self,level:number): (number)
	function getMipmapCount(self): (number)
	function getWidth(self): (number)
	function getWidth(self,level:number): (number)
end

declare class ImageData extends Data
	function encode(self,format:ImageFormat,filename:string): (FileData)
	function encode(self,outFile:string): ()
	function encode(self,outFile:string,format:ImageFormat): ()
	function getDimensions(self): (number,number)
	function getHeight(self): (number)
	function getPixel(self,x:number,y:number): (number,number,number,number)
	function getWidth(self): (number)
	function mapPixel(self,pixelFunction:(...any)->(...any),x:number,y:number,width:number,height:number): ()
	function paste(self,source:ImageData,dx:number,dy:number,sx:number,sy:number,sw:number,sh:number): ()
	function setPixel(self,x:number,y:number,r:number,g:number,b:number,a:number): ()
	function setPixel(self,x:number,y:number,color:{number}): ()
	function getFormat(self): (PixelFormat)
end

declare class Joystick extends Object
	function getAxes(self): (number,number,number)
	function getAxis(self,axis:number): (number)
	function getAxisCount(self): (number)
	function getButtonCount(self): (number)
	function getDeviceInfo(self): (number,number,number)
	function getGUID(self): (string)
	function getGamepadAxis(self,axis:GamepadAxis): (number)
	function getGamepadMapping(self,axis:GamepadAxis): (JoystickInputType,number,JoystickHat)
	function getGamepadMapping(self,button:GamepadButton): (JoystickInputType,number,JoystickHat)
	function getGamepadMappingString(self): (string)
	function getHat(self,hat:number): (JoystickHat)
	function getHatCount(self): (number)
	function getID(self): (number,number)
	function getName(self): (string)
	function getVibration(self): (number,number)
	function isConnected(self): (boolean)
	function isDown(self,buttonN:number): (boolean)
	function isGamepad(self): (boolean)
	function isGamepadDown(self,buttonN:GamepadButton): (boolean)
	function isVibrationSupported(self): (boolean)
	function setVibration(self,left:number,right:number): (boolean)
	function setVibration(self): (boolean)
	function setVibration(self,left:number,right:number,duration:number): (boolean)
end

declare class BezierCurve extends Object
	function evaluate(self,t:number): (number,number)
	function getControlPoint(self,i:number): (number,number)
	function getControlPointCount(self): (number)
	function getDegree(self): (number)
	function getDerivative(self): (BezierCurve)
	function getSegment(self,startpoint:number,endpoint:number): (BezierCurve)
	function insertControlPoint(self,x:number,y:number,i:number): ()
	function removeControlPoint(self,index:number): ()
	function render(self,depth:number): ({number})
	function renderSegment(self,startpoint:number,endpoint:number,depth:number): ({number})
	function rotate(self,angle:number,ox:number,oy:number): ()
	function scale(self,s:number,ox:number,oy:number): ()
	function setControlPoint(self,i:number,x:number,y:number): ()
	function translate(self,dx:number,dy:number): ()
end

declare class RandomGenerator extends Object
	function getSeed(self): (number,number)
	function getState(self): (string)
	function random(self): (number)
	function random(self,max:number): (number)
	function random(self,min:number,max:number): (number)
	function randomNormal(self,stddev:number,mean:number): (number)
	function setSeed(self,seed:number): ()
	function setSeed(self,low:number,high:number): ()
	function setState(self,state:string): ()
end

declare class Transform extends Object
	function apply(self,other:Transform): (Transform)
	function clone(self): (Transform)
	function getMatrix(self): (number,number,number,number,number,number,number,number,number,number,number,number,number,number,number,number)
	function inverse(self): (Transform)
	function inverseTransformPoint(self,localX:number,localY:number): (number,number)
	function isAffine2DTransform(self): (boolean)
	function reset(self): (Transform)
	function rotate(self,angle:number): (Transform)
	function scale(self,sx:number,sy:number): (Transform)
	function setMatrix(self,e1_1:number,e1_2:number,e1_3:number,e1_4:number,e2_1:number,e2_2:number,e2_3:number,e2_4:number,e3_1:number,e3_2:number,e3_3:number,e3_4:number,e4_1:number,e4_2:number,e4_3:number,e4_4:number): (Transform)
	function setMatrix(self,layout:MatrixLayout,e1_1:number,e1_2:number,e1_3:number,e1_4:number,e2_1:number,e2_2:number,e2_3:number,e2_4:number,e3_1:number,e3_2:number,e3_3:number,e3_4:number,e4_1:number,e4_2:number,e4_3:number,e4_4:number): (Transform)
	function setMatrix(self,layout:MatrixLayout,matrix:{number}): (Transform)
	function setMatrix(self,layout:MatrixLayout,matrix:{[any]: any}): (Transform)
	function setTransformation(self,x:number,y:number,angle:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number): (Transform)
	function shear(self,kx:number,ky:number): (Transform)
	function transformPoint(self,globalX:number,globalY:number): (number,number)
	function translate(self,dx:number,dy:number): (Transform)
end

declare class Cursor extends Object
	function getType(self): (CursorType)
end

declare class Body extends Object
	function applyAngularImpulse(self,impulse:number): ()
	function applyForce(self,fx:number,fy:number): ()
	function applyForce(self,fx:number,fy:number,x:number,y:number): ()
	function applyLinearImpulse(self,ix:number,iy:number): ()
	function applyLinearImpulse(self,ix:number,iy:number,x:number,y:number): ()
	function applyTorque(self,torque:number): ()
	function destroy(self): ()
	function getAngle(self): (number)
	function getAngularDamping(self): (number)
	function getAngularVelocity(self): (number)
	function getContacts(self): ({Contact})
	function getFixtures(self): ({Fixture})
	function getGravityScale(self): (number)
	function getInertia(self): (number)
	function getJoints(self): ({Joint})
	function getLinearDamping(self): (number)
	function getLinearVelocity(self): (number,number)
	function getLinearVelocityFromLocalPoint(self,x:number,y:number): (number,number)
	function getLinearVelocityFromWorldPoint(self,x:number,y:number): (number,number)
	function getLocalCenter(self): (number,number)
	function getLocalPoint(self,worldX:number,worldY:number): (number,number)
	function getLocalPoints(self,x1:number,y1:number,x2:number,y2:number,...:number): (number,number,number,number,number)
	function getLocalVector(self,worldX:number,worldY:number): (number,number)
	function getMass(self): (number)
	function getMassData(self): (number,number,number,number)
	function getPosition(self): (number,number)
	function getTransform(self): (number,number,number)
	function getType(self): (BodyType)
	function getUserData(self): (any)
	function getWorld(self): (World)
	function getWorldCenter(self): (number,number)
	function getWorldPoint(self,localX:number,localY:number): (number,number)
	function getWorldPoints(self,x1:number,y1:number,x2:number,y2:number): (number,number,number,number)
	function getWorldVector(self,localX:number,localY:number): (number,number)
	function getX(self): (number)
	function getY(self): (number)
	function isActive(self): (boolean)
	function isAwake(self): (boolean)
	function isBullet(self): (boolean)
	function isDestroyed(self): (boolean)
	function isFixedRotation(self): (boolean)
	function isSleepingAllowed(self): (boolean)
	function isTouching(self,otherbody:Body): (boolean)
	function resetMassData(self): ()
	function setActive(self,active:boolean): ()
	function setAngle(self,angle:number): ()
	function setAngularDamping(self,damping:number): ()
	function setAngularVelocity(self,w:number): ()
	function setAwake(self,awake:boolean): ()
	function setBullet(self,status:boolean): ()
	function setFixedRotation(self,isFixed:boolean): ()
	function setGravityScale(self,scale:number): ()
	function setInertia(self,inertia:number): ()
	function setLinearDamping(self,ld:number): ()
	function setLinearVelocity(self,x:number,y:number): ()
	function setMass(self,mass:number): ()
	function setMassData(self,x:number,y:number,mass:number,inertia:number): ()
	function setPosition(self,x:number,y:number): ()
	function setSleepingAllowed(self,allowed:boolean): ()
	function setTransform(self,x:number,y:number,angle:number): ()
	function setType(self,type:BodyType): ()
	function setUserData(self,value:any): ()
	function setX(self,x:number): ()
	function setY(self,y:number): ()
end

declare class Shape extends Object
	function computeAABB(self,tx:number,ty:number,tr:number,childIndex:number): (number,number,number,number)
	function computeMass(self,density:number): (number,number,number,number)
	function getChildCount(self): (number)
	function getRadius(self): (number)
	function getType(self): (ShapeType)
	function rayCast(self,x1:number,y1:number,x2:number,y2:number,maxFraction:number,tx:number,ty:number,tr:number,childIndex:number): (number,number,number)
	function testPoint(self,tx:number,ty:number,tr:number,x:number,y:number): (boolean)
end

declare class ChainShape extends Shape
	function getChildEdge(self,index:number): (EdgeShape)
	function getNextVertex(self): (number,number)
	function getPoint(self,index:number): (number,number)
	function getPoints(self): (number,number,number,number)
	function getPreviousVertex(self): (number,number)
	function getVertexCount(self): (number)
	function setNextVertex(self,x:number,y:number): ()
	function setPreviousVertex(self,x:number,y:number): ()
end

declare class CircleShape extends Shape
	function getPoint(self): (number,number)
	function getRadius(self): (number)
	function setPoint(self,x:number,y:number): ()
	function setRadius(self,radius:number): ()
end

declare class Contact extends Object
	function getChildren(self): (number,number)
	function getFixtures(self): (Fixture,Fixture)
	function getFriction(self): (number)
	function getNormal(self): (number,number)
	function getPositions(self): (number,number,number,number)
	function getRestitution(self): (number)
	function isEnabled(self): (boolean)
	function isTouching(self): (boolean)
	function resetFriction(self): ()
	function resetRestitution(self): ()
	function setEnabled(self,enabled:boolean): ()
	function setFriction(self,friction:number): ()
	function setRestitution(self,restitution:number): ()
end

declare class Joint extends Object
	function destroy(self): ()
	function getAnchors(self): (number,number,number,number)
	function getBodies(self): (Body,Body)
	function getCollideConnected(self): (boolean)
	function getReactionForce(self,x:number): (number,number)
	function getReactionTorque(self,invdt:number): (number)
	function getType(self): (JointType)
	function getUserData(self): (any)
	function isDestroyed(self): (boolean)
	function setUserData(self,value:any): ()
end

declare class DistanceJoint extends Joint
	function getDampingRatio(self): (number)
	function getFrequency(self): (number)
	function getLength(self): (number)
	function setDampingRatio(self,ratio:number): ()
	function setFrequency(self,Hz:number): ()
	function setLength(self,l:number): ()
end

declare class EdgeShape extends Shape
	function getNextVertex(self): (number,number)
	function getPoints(self): (number,number,number,number)
	function getPreviousVertex(self): (number,number)
	function setNextVertex(self,x:number,y:number): ()
	function setPreviousVertex(self,x:number,y:number): ()
end

declare class Fixture extends Object
	function destroy(self): ()
	function getBody(self): (Body)
	function getBoundingBox(self,index:number): (number,number,number,number)
	function getCategory(self): (number)
	function getDensity(self): (number)
	function getFilterData(self): (number,number,number)
	function getFriction(self): (number)
	function getGroupIndex(self): (number)
	function getMask(self): (number)
	function getMassData(self): (number,number,number,number)
	function getRestitution(self): (number)
	function getShape(self): (Shape)
	function getUserData(self): (any)
	function isDestroyed(self): (boolean)
	function isSensor(self): (boolean)
	function rayCast(self,x1:number,y1:number,x2:number,y2:number,maxFraction:number,childIndex:number): (number,number,number)
	function setCategory(self,...:number): ()
	function setDensity(self,density:number): ()
	function setFilterData(self,categories:number,mask:number,group:number): ()
	function setFriction(self,friction:number): ()
	function setGroupIndex(self,group:number): ()
	function setMask(self,...:number): ()
	function setRestitution(self,restitution:number): ()
	function setSensor(self,sensor:boolean): ()
	function setUserData(self,value:any): ()
	function testPoint(self,x:number,y:number): (boolean)
end

declare class FrictionJoint extends Joint
	function getMaxForce(self): (number)
	function getMaxTorque(self): (number)
	function setMaxForce(self,maxForce:number): ()
	function setMaxTorque(self,torque:number): ()
end

declare class GearJoint extends Joint
	function getJoints(self): (Joint,Joint)
	function getRatio(self): (number)
	function setRatio(self,ratio:number): ()
end

declare class MotorJoint extends Joint
	function getAngularOffset(self): (number)
	function getLinearOffset(self): (number,number)
	function setAngularOffset(self,angleoffset:number): ()
	function setLinearOffset(self,x:number,y:number): ()
end

declare class MouseJoint extends Joint
	function getDampingRatio(self): (number)
	function getFrequency(self): (number)
	function getMaxForce(self): (number)
	function getTarget(self): (number,number)
	function setDampingRatio(self,ratio:number): ()
	function setFrequency(self,freq:number): ()
	function setMaxForce(self,f:number): ()
	function setTarget(self,x:number,y:number): ()
end

declare class PolygonShape extends Shape
	function getPoints(self): (number,number,number,number)
end

declare class PrismaticJoint extends Joint
	function areLimitsEnabled(self): (boolean)
	function getAxis(self): (number,number)
	function getJointSpeed(self): (number)
	function getJointTranslation(self): (number)
	function getLimits(self): (number,number)
	function getLowerLimit(self): (number)
	function getMaxMotorForce(self): (number)
	function getMotorForce(self,invdt:number): (number)
	function getMotorSpeed(self): (number)
	function getReferenceAngle(self): (number)
	function getUpperLimit(self): (number)
	function isMotorEnabled(self): (boolean)
	function setLimits(self,lower:number,upper:number): ()
	function setLimitsEnabled(self): (boolean)
	function setLowerLimit(self,lower:number): ()
	function setMaxMotorForce(self,f:number): ()
	function setMotorEnabled(self,enable:boolean): ()
	function setMotorSpeed(self,s:number): ()
	function setUpperLimit(self,upper:number): ()
end

declare class PulleyJoint extends Joint
	function getConstant(self): (number)
	function getGroundAnchors(self): (number,number,number,number)
	function getLengthA(self): (number)
	function getLengthB(self): (number)
	function getMaxLengths(self): (number,number)
	function getRatio(self): (number)
	function setConstant(self,length:number): ()
	function setMaxLengths(self,max1:number,max2:number): ()
	function setRatio(self,ratio:number): ()
end

declare class RevoluteJoint extends Joint
	function areLimitsEnabled(self): (boolean)
	function getJointAngle(self): (number)
	function getJointSpeed(self): (number)
	function getLimits(self): (number,number)
	function getLowerLimit(self): (number)
	function getMaxMotorTorque(self): (number)
	function getMotorSpeed(self): (number)
	function getMotorTorque(self): (number)
	function getReferenceAngle(self): (number)
	function getUpperLimit(self): (number)
	function hasLimitsEnabled(self): (boolean)
	function isMotorEnabled(self): (boolean)
	function setLimits(self,lower:number,upper:number): ()
	function setLimitsEnabled(self,enable:boolean): ()
	function setLowerLimit(self,lower:number): ()
	function setMaxMotorTorque(self,f:number): ()
	function setMotorEnabled(self,enable:boolean): ()
	function setMotorSpeed(self,s:number): ()
	function setUpperLimit(self,upper:number): ()
end

declare class RopeJoint extends Joint
	function getMaxLength(self): (number)
	function setMaxLength(self,maxLength:number): ()
end

declare class WeldJoint extends Joint
	function getDampingRatio(self): (number)
	function getFrequency(self): (number)
	function getReferenceAngle(self): (number)
	function setDampingRatio(self,ratio:number): ()
	function setFrequency(self,freq:number): ()
end

declare class WheelJoint extends Joint
	function getAxis(self): (number,number)
	function getJointSpeed(self): (number)
	function getJointTranslation(self): (number)
	function getMaxMotorTorque(self): (number)
	function getMotorSpeed(self): (number)
	function getMotorTorque(self,invdt:number): (number)
	function getSpringDampingRatio(self): (number)
	function getSpringFrequency(self): (number)
	function isMotorEnabled(self): (boolean)
	function setMaxMotorTorque(self,maxTorque:number): ()
	function setMotorEnabled(self,enable:boolean): ()
	function setMotorSpeed(self,speed:number): ()
	function setSpringDampingRatio(self,ratio:number): ()
	function setSpringFrequency(self,freq:number): ()
end

declare class World extends Object
	function destroy(self): ()
	function getBodies(self): ({Body})
	function getBodyCount(self): (number)
	function getCallbacks(self): ((...any)->(...any),(...any)->(...any),(...any)->(...any),(...any)->(...any))
	function getContactCount(self): (number)
	function getContactFilter(self): ((...any)->(...any))
	function getContacts(self): ({Contact})
	function getGravity(self): (number,number)
	function getJointCount(self): (number)
	function getJoints(self): ({Joint})
	function isDestroyed(self): (boolean)
	function isLocked(self): (boolean)
	function isSleepingAllowed(self): (boolean)
	function queryBoundingBox(self,topLeftX:number,topLeftY:number,bottomRightX:number,bottomRightY:number,callback:(...any)->(...any)): ()
	function rayCast(self,x1:number,y1:number,x2:number,y2:number,callback:(...any)->(...any)): ()
	function setCallbacks(self,beginContact:(...any)->(...any),endContact:(...any)->(...any),preSolve:(...any)->(...any),postSolve:(...any)->(...any)): ()
	function setContactFilter(self,filter:(...any)->(...any)): ()
	function setGravity(self,x:number,y:number): ()
	function setSleepingAllowed(self,allow:boolean): ()
	function translateOrigin(self,x:number,y:number): ()
	function update(self,dt:number,velocityiterations:number,positioniterations:number): ()
end

declare class SoundData extends Data
	function getBitDepth(self): (number)
	function getChannelCount(self): (number)
	function getDuration(self): (number)
	function getSample(self,i:number): (number)
	function getSample(self,i:number,channel:number): (number)
	function getSampleCount(self): (number)
	function getSampleRate(self): (number)
	function setSample(self,i:number,sample:number): ()
	function setSample(self,i:number,channel:number,sample:number): ()
end

declare class Decoder extends Object
	function clone(self): (Decoder)
	function decode(self): (SoundData)
	function getBitDepth(self): (number)
	function getChannelCount(self): (number)
	function getDuration(self): (number)
	function getSampleRate(self): (number)
	function seek(self,offset:number): ()
end



declare class Channel extends Object
	function clear(self): ()
	function demand(self): (Variant)
	function demand(self,timeout:number): (Variant)
	function getCount(self): (number)
	function hasRead(self,id:number): (boolean)
	function peek(self): (Variant)
	function performAtomic(self,func:(...any)->(...any),...:any): (any,any)
	function pop(self): (Variant)
	function push(self,value:Variant): (number)
	function supply(self,value:Variant): (boolean)
	function supply(self,value:Variant,timeout:number): (boolean)
end

declare class Thread extends Object
	function getError(self): (string)
	function isRunning(self): (boolean)
	function start(self): ()
	function start(self,...:Variant): ()
	function wait(self): ()
end

declare class VideoStream extends Object
	function getFilename(self): (string)
	function isPlaying(self): (boolean)
	function pause(self): ()
	function play(self): ()
	function rewind(self): ()
	function seek(self,offset:number): ()
	function tell(self): (number)
end

type getModeReturn = {
	fullscreen:boolean,
	display:number,
	minwidth:number,
	minheight:number,
	highdpi:boolean,
	refreshrate:number,
	x:number,
	y:number,
	srgb:boolean,
	fullscreentype:'desktop'|'exclusive'|'normal',
	vsync:boolean,
	msaa:number,
	resizable:boolean,
	borderless:boolean,
	centered:boolean,
}

declare love: {
	getVersion: () -> (number,number,number,string),
	hasDeprecationOutput: () -> (boolean),
	isVersionCompatible: (version:string) -> (boolean),
	isVersionCompatible: (major:number,minor:number,revision:number) -> (boolean),
	setDeprecationOutput: (enable:boolean) -> (),
	conf: (t:{identity:string?,appendidentity:boolean?,version:string?,console:boolean?,accelerometerjoystick:boolean?,externalstorage:boolean?,gammacorrect:boolean?,audio:{mic:boolean?,mixwithsystem:boolean?,},window:{title:string?,icon:string?,width:number?,height:number?,borderless:boolean?,resizable:boolean?,minwidth:number?,minheight:number?,fullscreen:boolean?,fullscreentype:string?,usedpiscale:boolean?,vsync:number?,depth:number?,stencil:number?,msaa:number?,display:number?,highdpi:boolean?,x:number?,y:number?,},modules:{audio:boolean?,event:boolean?,graphics:boolean?,image:boolean?,joystick:boolean?,keyboard:boolean?,math:boolean?,mouse:boolean?,physics:boolean?,sound:boolean?,system:boolean?,timer:boolean?,touch:boolean?,video:boolean?,window:boolean?,thread:boolean?,},}) -> (),
	directorydropped: (path:string) -> (),
	displayrotated: (index:number,orientation:DisplayOrientation) -> (),
	draw: () -> (),
	errorhandler: (msg:string) -> ((...any)->(...any)),
	filedropped: (file:DroppedFile) -> (),
	focus: (focus:boolean) -> (),
	gamepadaxis: (joystick:Joystick,axis:GamepadAxis,value:number) -> (),
	gamepadpressed: (joystick:Joystick,button:GamepadButton) -> (),
	gamepadreleased: (joystick:Joystick,button:GamepadButton) -> (),
	joystickadded: (joystick:Joystick) -> (),
	joystickaxis: (joystick:Joystick,axis:number,value:number) -> (),
	joystickhat: (joystick:Joystick,hat:number,direction:JoystickHat) -> (),
	joystickpressed: (joystick:Joystick,button:number) -> (),
	joystickreleased: (joystick:Joystick,button:number) -> (),
	joystickremoved: (joystick:Joystick) -> (),
	keypressed: (key:KeyConstant,scancode:Scancode,isrepeat:boolean) -> (),
	keypressed: (key:KeyConstant,isrepeat:boolean) -> (),
	keyreleased: (key:KeyConstant,scancode:Scancode) -> (),
	load: (arg:{[any]: any},unfilteredArg:{[any]: any}) -> (),
	lowmemory: () -> (),
	mousefocus: (focus:boolean) -> (),
	mousemoved: (x:number,y:number,dx:number,dy:number,istouch:boolean) -> (),
	mousepressed: (x:number,y:number,button:number,istouch:boolean,presses:number) -> (),
	mousereleased: (x:number,y:number,button:number,istouch:boolean,presses:number) -> (),
	quit: () -> (boolean),
	resize: (w:number,h:number) -> (),
	run: () -> ((...any)->(...any)),
	textedited: (text:string,start:number,length:number) -> (),
	textinput: (text:string) -> (),
	threaderror: (thread:Thread,errorstr:string) -> (),
	touchmoved: (id:unknown,x:number,y:number,dx:number,dy:number,pressure:number) -> (),
	touchpressed: (id:unknown,x:number,y:number,dx:number,dy:number,pressure:number) -> (),
	touchreleased: (id:unknown,x:number,y:number,dx:number,dy:number,pressure:number) -> (),
	update: (dt:number) -> (),
	visible: (visible:boolean) -> (),
	wheelmoved: (x:number,y:number) -> (),
} & {
	audio: {
		getActiveEffects: (() -> ({string})),
		getActiveSourceCount: (() -> (number)),
		getDistanceModel: (() -> (DistanceModel)),
		getDopplerScale: (() -> (number)),
		getEffect: ((name:string) -> ({[any]: any})),
		getMaxSceneEffects: (() -> (number)),
		getMaxSourceEffects: (() -> (number)),
		getOrientation: (() -> (number,number,number,number,number,number)),
		getPosition: (() -> (number,number,number)),
		getRecordingDevices: (() -> ({RecordingDevice})),
		getVelocity: (() -> (number,number,number)),
		getVolume: (() -> (number)),
		isEffectsSupported: (() -> (boolean)),
		newQueueableSource: ((samplerate:number,bitdepth:number,channels:number,buffercount:number) -> (Source)),
		newSource: ((filename:string,type:SourceType) -> (Source))&((file:File,type:SourceType) -> (Source))&((decoder:Decoder,type:SourceType) -> (Source))&((data:FileData,type:SourceType) -> (Source))&((data:SoundData) -> (Source)),
		pause: (() -> ({Source}))&((source:Source,...Source) -> ())&((sources:{Source}) -> ()),
		play: ((source:Source) -> ())&((sources:{Source}) -> ())&((source1:Source,source2:Source,...Source) -> ()),
		setDistanceModel: ((model:DistanceModel) -> ()),
		setDopplerScale: ((scale:number) -> ()),
		setEffect: ((name:string,settings:{type:EffectType,volume:number}?) -> (boolean))&((name:string,enabled:boolean) -> (boolean)),
		setMixWithSystem: ((mix:boolean) -> (boolean)),
		setOrientation: ((fx:number, fy:number, fz:number, ux:number, uy:number, uz:number) -> ()),
		setPosition: ((x:number,y:number,z:number) -> ()),
		setVelocity: ((x:number,y:number,z:number) -> ()),
		setVolume: ((volume:number) -> ()),
		stop: (() -> ())&((source:Source) -> ())&((source1:Source,source2:Source,...Source) -> ())&((sources:{Source}) -> ()),
	},
} & {
	data: {
		compress: ((container:ContainerType,format:CompressedDataFormat,rawstring:string,level:number) -> (CompressedData|string))&((container:ContainerType,format:CompressedDataFormat,data:Data,level:number) -> (CompressedData|string)),
		decode: ((container:ContainerType,format:EncodeFormat,sourceString:string) -> (ByteData|string))&((container:ContainerType,format:EncodeFormat,sourceData:Data) -> (ByteData|string)),
		decompress: ((container:ContainerType,compressedData:CompressedData) -> (Data|string))&((container:ContainerType,format:CompressedDataFormat,compressedString:string) -> (Data|string))&((container:ContainerType,format:CompressedDataFormat,data:Data) -> (Data|string)),
		encode: ((container:ContainerType,format:EncodeFormat,sourceString:string,linelength:number) -> (ByteData|string))&((container:ContainerType,format:EncodeFormat,sourceData:Data,linelength:number) -> (ByteData|string)),
		getPackedSize: ((format:string) -> (number)),
		hash: ((hashFunction:HashFunction,string:string) -> (string))&((hashFunction:HashFunction,data:Data) -> (string)),
		newByteData: ((datastring:string) -> (ByteData))&((Data:Data,offset:number,size:number) -> (ByteData))&((size:number) -> (ByteData)),
		newDataView: ((data:Data,offset:number,size:number) -> (Data)),
		pack: ((container:ContainerType,format:string,v1:number|boolean|string,...number|boolean|string) -> (Data|string)),
		unpack: ((format:string,datastring:string,pos:number) -> (number|boolean|string,...number|boolean|string))&((format:string,data:Data,pos:number) -> (number|boolean|string,...number|boolean|string)),
	},
} & {
	event: {
		clear: (() -> ()),
		poll: (() -> ((...any)->(...any))),
		pump: (() -> ()),
		push: ((n:Event,a:Variant,b:Variant,c:Variant,d:Variant,e:Variant,f:Variant,...Variant) -> ()),
		quit: ((exitstatus:number) -> ())&(('restart'|nil) -> ()),
		wait: (() -> (Event,Variant,Variant,Variant,Variant,Variant,Variant,...Variant)),
	},
	filesystem: {
		append: ((name:string,data:string,size:number) -> (boolean,string))&((name:string,data:Data,size:number) -> (boolean,string)),
		areSymlinksEnabled: (() -> (boolean)),
		createDirectory: ((name:string) -> (boolean)),
		getAppdataDirectory: (() -> (string)),
		getCRequirePath: (() -> (string)),
		getDirectoryItems: ((dir:string) -> ({string}))&((dir:string,callback:(...any)->(...any)) -> ({[any]: any})),
		getIdentity: (() -> (string)),
		getInfo: ((path:string,filtertype:FileType) -> ({type:FileType,size:number,modtime:number,}))&((path:string,info:{[any]: any}) -> ({type:FileType,size:number,modtime:number,}))&((path:string,filtertype:FileType,info:{[any]: any}) -> ({type:FileType,size:number,modtime:number,})),
		getRealDirectory: ((filepath:string) -> (string)),
		getRequirePath: (() -> (string)),
		getSaveDirectory: (() -> (string)),
		getSource: (() -> (string)),
		getSourceBaseDirectory: (() -> (string)),
		getUserDirectory: (() -> (string)),
		getWorkingDirectory: (() -> (string)),
		init: ((appname:string) -> ()),
		isFused: (() -> (boolean)),
		lines: ((name:string) -> ((...any)->(...any))),
		load: ((name:string) -> ((...any)->(...any),string)),
		mount: ((archive:string,mountpoint:string,appendToPath:boolean) -> (boolean))&((filedata:FileData,mountpoint:string,appendToPath:boolean) -> (boolean))&((data:Data,archivename:string,mountpoint:string,appendToPath:boolean) -> (boolean)),
		newFile: ((filename:string) -> (File))&((filename:string,mode:FileMode) -> (File,string)),
		newFileData: ((contents:string,name:string) -> (FileData))&((originaldata:Data,name:string) -> (FileData))&((filepath:string) -> (FileData,string)),
		read: ((name:string,size:number) -> (string,number,nil,string))&((container:ContainerType,name:string,size:number) -> (FileData|string,number,nil,string)),
		remove: ((name:string) -> (boolean)),
		setCRequirePath: ((paths:string) -> ()),
		setIdentity: ((name:string) -> ())&((name:string) -> ()),
		setRequirePath: ((paths:string) -> ()),
		setSource: ((path:string) -> ()),
		setSymlinksEnabled: ((enable:boolean) -> ()),
		unmount: ((archive:string) -> (boolean)),
		write: ((name:string,data:string,size:number) -> (boolean,string))&((name:string,data:Data,size:number) -> (boolean,string)),
	},
} & {
	font: {
		newBMFontRasterizer: ((imageData:ImageData,glyphs:string,dpiscale:number) -> (Rasterizer))&((fileName:string,glyphs:string,dpiscale:number) -> (Rasterizer)),
		newGlyphData: ((rasterizer:Rasterizer,glyph:number) -> ()),
		newImageRasterizer: ((imageData:ImageData,glyphs:string,extraSpacing:number,dpiscale:number) -> (Rasterizer)),
		newRasterizer: ((filename:string) -> (Rasterizer))&((data:FileData) -> (Rasterizer))&((size:number,hinting:HintingMode,dpiscale:number) -> (Rasterizer))&((fileName:string,size:number,hinting:HintingMode,dpiscale:number) -> (Rasterizer))&((fileData:FileData,size:number,hinting:HintingMode,dpiscale:number) -> (Rasterizer))&((imageData:ImageData,glyphs:string,dpiscale:number) -> (Rasterizer))&((fileName:string,glyphs:string,dpiscale:number) -> (Rasterizer)),
		newTrueTypeRasterizer: ((size:number,hinting:HintingMode,dpiscale:number) -> (Rasterizer))&((fileName:string,size:number,hinting:HintingMode,dpiscale:number) -> (Rasterizer))&((fileData:FileData,size:number,hinting:HintingMode,dpiscale:number) -> (Rasterizer)),
	},
} & {
	graphics: {
		applyTransform: ((transform:Transform) -> ()),
		arc: ((drawmode:DrawMode,x:number,y:number,radius:number,angle1:number,angle2:number,segments:number) -> ())&((drawmode:DrawMode,arctype:ArcType,x:number,y:number,radius:number,angle1:number,angle2:number,segments:number) -> ()),
		captureScreenshot: ((filename:string) -> ())&((callback:(...any)->(...any)) -> ())&((channel:Channel) -> ()),
		circle: ((mode:DrawMode,x:number,y:number,radius:number) -> ())&((mode:DrawMode,x:number,y:number,radius:number,segments:number) -> ()),
		clear: (() -> ())&((r:number,g:number,b:number,a:number,clearstencil:boolean,cleardepth:boolean) -> ())&((color:{number},cleardepth:boolean) -> ())&((clearcolor:boolean,clearstencil:boolean,cleardepth:boolean) -> ()),
		discard: ((discardcolor:boolean,discardstencil:boolean) -> ())&((discardcolors:{boolean},discardstencil:boolean) -> ()),
		draw: ((drawable:Drawable,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> ())&((texture:Texture,quad:Quad,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> ())&((drawable:Drawable,transform:Transform) -> ())&((texture:Texture,quad:Quad,transform:Transform) -> ()),
		drawInstanced: ((mesh:Mesh,instancecount:number,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> ())&((mesh:Mesh,instancecount:number,transform:Transform) -> ()),
		drawLayer: ((texture:Texture,layerindex:number,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> ())&((texture:Texture,layerindex:number,quad:Quad,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> ())&((texture:Texture,layerindex:number,transform:Transform) -> ())&((texture:Texture,layerindex:number,quad:Quad,transform:Transform) -> ()),
		ellipse: ((mode:DrawMode,x:number,y:number,radiusx:number,radiusy:number) -> ())&((mode:DrawMode,x:number,y:number,radiusx:number,radiusy:number,segments:number) -> ()),
		flushBatch: (() -> ()),
		getBackgroundColor: (() -> (number,number,number,number)),
		getBlendMode: (() -> (BlendMode,BlendAlphaMode)),
		getCanvas: (() -> (Canvas)),
		getCanvasFormats: (() -> ({[PixelFormat]:boolean}))&((readable:boolean) -> ({[PixelFormat]:boolean})),
		getColor: (() -> (number,number,number,number)),
		getColorMask: (() -> (boolean,boolean,boolean,boolean)),
		getDPIScale: (() -> (number)),
		getDefaultFilter: (() -> (FilterMode,FilterMode,number)),
		getDepthMode: (() -> (CompareMode,boolean)),
		getDimensions: (() -> (number,number)),
		getFont: (() -> (Font)),
		getFrontFaceWinding: (() -> (VertexWinding)),
		getHeight: (() -> (number)),
		getImageFormats: (() -> ({[PixelFormat]:boolean})),
		getLineJoin: (() -> (LineJoin)),
		getLineStyle: (() -> (LineStyle)),
		getLineWidth: (() -> (number)),
		getMeshCullMode: (() -> (CullMode)),
		getPixelDimensions: (() -> (number,number)),
		getPixelHeight: (() -> (number)),
		getPixelWidth: (() -> (number)),
		getPointSize: (() -> (number)),
		getRendererInfo: (() -> (string,string,string,string)),
		getScissor: (() -> (number,number,number,number)),
		getShader: (() -> (Shader)),
		getStackDepth: (() -> (number)),
		getStats: (() -> ({drawcalls:number,canvasswitches:number,texturememory:number,images:number,canvases:number,fonts:number,shaderswitches:number,drawcallsbatched:number,}))&((stats:{[any]: any}) -> ({drawcalls:number,canvasswitches:number,texturememory:number,images:number,canvases:number,fonts:number,shaderswitches:number,drawcallsbatched:number,})),
		getStencilTest: (() -> (CompareMode,number)),
		getSupported: (() -> ({[GraphicsFeature]:boolean})),
		getSystemLimits: (() -> ({[GraphicsLimit]:number})),
		getTextureTypes: (() -> ({[TextureType]:boolean})),
		getWidth: (() -> (number)),
		intersectScissor: ((x:number,y:number,width:number,height:number) -> ()),
		inverseTransformPoint: ((screenX:number,screenY:number) -> (number,number)),
		isActive: (() -> (boolean)),
		isGammaCorrect: (() -> (boolean)),
		isWireframe: (() -> (boolean)),
		line: ((x1:number,y1:number,x2:number,y2:number,...number) -> ())&((points:{number}) -> ()),
		newArrayImage: ((slices:{[any]: any},settings:{mipmaps:boolean?,linear:boolean?,dpiscale:number?,}) -> (Image)),
		newCanvas: (() -> (Canvas))&((width:number,height:number) -> (Canvas))&((width:number,height:number,settings:{type:TextureType?,format:PixelFormat?,readable:boolean,msaa:number?,dpiscale:number?,mipmaps:MipmapMode?,}) -> (Canvas))&((width:number,height:number,layers:number,settings:{type:TextureType?,format:PixelFormat?,readable:boolean,msaa:number?,dpiscale:number?,mipmaps:MipmapMode?,}) -> (Canvas)),
		newCubeImage: ((filename:string,settings:{mipmaps:boolean?,linear:boolean?,}) -> (Image))&((faces:{[any]: any},settings:{mipmaps:boolean?,linear:boolean?,}) -> (Image)),
		newFont: ((filename:string) -> (Font))&((filename:string,size:number,hinting:HintingMode,dpiscale:number) -> (Font))&((filename:string,imagefilename:string) -> (Font))&((size:number,hinting:HintingMode,dpiscale:number) -> (Font)),
		newImage: ((filename:string,settings:{dpiscale:number?,linear:boolean?,mipmaps:boolean?,}) -> (Image))&((fileData:FileData,settings:{dpiscale:number?,linear:boolean?,mipmaps:boolean?,}) -> (Image))&((imageData:ImageData,settings:{dpiscale:number?,linear:boolean?,mipmaps:boolean?,}) -> (Image))&((compressedImageData:CompressedImageData,settings:{dpiscale:number?,linear:boolean?,mipmaps:boolean?,}) -> (Image)),
		newImageFont: ((filename:string,glyphs:string) -> (Font))&((imageData:ImageData,glyphs:string) -> (Font))&((filename:string,glyphs:string,extraspacing:number) -> (Font)),
		newMesh: ((vertices:{number},mode:MeshDrawMode,usage:SpriteBatchUsage) -> (Mesh))&((vertexcount:number,mode:MeshDrawMode,usage:SpriteBatchUsage) -> (Mesh))&((vertexformat:{attribute:{[any]: any}},vertices:{attributecomponent:number},mode:MeshDrawMode,usage:SpriteBatchUsage) -> (Mesh))&((vertexformat:{attribute:{[any]: any}},vertexcount:number,mode:MeshDrawMode,usage:SpriteBatchUsage) -> (Mesh))&((vertexcount:number,texture:Texture,mode:MeshDrawMode) -> (Mesh)),
		newParticleSystem: ((image:Image,buffer:number) -> (ParticleSystem))&((texture:Texture,buffer:number) -> (ParticleSystem)),
		newQuad: ((x:number,y:number,width:number,height:number,sw:number,sh:number) -> (Quad))&((x:number,y:number,width:number,height:number,texture:Texture) -> (Quad)),
		newShader: ((code:string) -> (Shader))&((pixelcode:string,vertexcode:string) -> (Shader)),
		newSpriteBatch: ((image:Image,maxsprites:number) -> (SpriteBatch))&((image:Image,maxsprites:number,usage:SpriteBatchUsage) -> (SpriteBatch))&((texture:Texture,maxsprites:number,usage:SpriteBatchUsage) -> (SpriteBatch)),
		newText: ((font:Font,textstring:string) -> (Text))&((font:Font,coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string}) -> (Text)),
		newVideo: ((filename:string) -> (Video))&((videostream:VideoStream) -> (Video))&((filename:string,settings:{audio:boolean?,dpiscale:number?,}) -> (Video))&((filename:string,loadaudio:boolean) -> (Video))&((videostream:VideoStream,loadaudio:boolean) -> (Video)),
		newVolumeImage: ((layers:{[any]: any},settings:{mipmaps:boolean?,linear:boolean?,}) -> (Image)),
		origin: (() -> ()),
		points: ((x:number,y:number,...number) -> ())&((points:{number}) -> ())&((points:{point:{[any]: any}}) -> ()),
		polygon: ((mode:DrawMode,...number) -> ())&((mode:DrawMode,vertices:{number}) -> ()),
		pop: (() -> ()),
		present: (() -> ()),
		print: ((text:string,x:number,y:number,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> ())&((coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string},x:number,y:number,angle:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> ())&((text:string,transform:Transform) -> ())&((coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string},transform:Transform) -> ())&((text:string,font:Font,transform:Transform) -> ())&((coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string},font:Font,transform:Transform) -> ()),
		printf: ((text:string,x:number,y:number,limit:number,align:AlignMode,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> ())&((text:string,font:Font,x:number,y:number,limit:number,align:AlignMode,r:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> ())&((text:string,transform:Transform,limit:number,align:AlignMode) -> ())&((text:string,font:Font,transform:Transform,limit:number,align:AlignMode) -> ())&((coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string},x:number,y:number,limit:number,align:AlignMode,angle:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> ())&((coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string},font:Font,x:number,y:number,limit:number,align:AlignMode,angle:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> ())&((coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string},transform:Transform,limit:number,align:AlignMode) -> ())&((coloredtext:{color1:{[any]: any},string1:string,color2:{[any]: any},string2:string},font:Font,transform:Transform,limit:number,align:AlignMode) -> ()),
		push: (() -> ())&((stack:StackType) -> ()),
		rectangle: ((mode:DrawMode,x:number,y:number,width:number,height:number) -> ())&((mode:DrawMode,x:number,y:number,width:number,height:number,rx:number,ry:number,segments:number) -> ()),
		replaceTransform: ((transform:Transform) -> ()),
		reset: (() -> ()),
		rotate: ((angle:number) -> ()),
		scale: ((sx:number,sy:number) -> ()),
		setBackgroundColor: ((red:number,green:number,blue:number,alpha:number) -> ())&((rgba:{number}) -> ()),
		setBlendMode: ((mode:BlendMode) -> ())&((mode:BlendMode,alphamode:BlendAlphaMode) -> ()),
		setCanvas: ((canvas:Canvas,mipmap:number) -> ())&(() -> ())&((canvas1:Canvas,canvas2:Canvas,...Canvas) -> ())&((canvas:Canvas,slice:number,mipmap:number) -> ())&((setup:{number}) -> ()),
		setColor: ((red:number,green:number,blue:number,alpha:number) -> ())&((rgba:{number}) -> ()),
		setColorMask: ((red:boolean,green:boolean,blue:boolean,alpha:boolean) -> ())&(() -> ()),
		setDefaultFilter: ((min:FilterMode,mag:FilterMode,anisotropy:number) -> ()),
		setDepthMode: ((comparemode:CompareMode,write:boolean) -> ())&(() -> ()),
		setFont: ((font:Font) -> ()),
		setFrontFaceWinding: ((winding:VertexWinding) -> ()),
		setLineJoin: ((join:LineJoin) -> ()),
		setLineStyle: ((style:LineStyle) -> ()),
		setLineWidth: ((width:number) -> ()),
		setMeshCullMode: ((mode:CullMode) -> ()),
		setNewFont: ((size:number) -> (Font))&((filename:string,size:number) -> (Font))&((file:File,size:number) -> (Font))&((data:Data,size:number) -> (Font))&((rasterizer:Rasterizer) -> (Font)),
		setPointSize: ((size:number) -> ()),
		setScissor: ((x:number,y:number,width:number,height:number) -> ())&(() -> ()),
		setShader: ((shader:Shader) -> ())&(() -> ()),
		setStencilTest: ((comparemode:CompareMode,comparevalue:number) -> ())&(() -> ()),
		setWireframe: ((enable:boolean) -> ()),
		shear: ((kx:number,ky:number) -> ()),
		stencil: ((stencilfunction:(...any)->(...any),action:StencilAction,value:number,keepvalues:boolean) -> ()),
		transformPoint: ((globalX:number,globalY:number) -> (number,number)),
		translate: ((dx:number,dy:number) -> ()),
		validateShader: ((gles:boolean,code:string) -> (boolean,string))&((gles:boolean,pixelcode:string,vertexcode:string) -> (boolean,string)),
	},
} & {
	image: {
		isCompressed: ((filename:string) -> (boolean))&((fileData:FileData) -> (boolean)),
		newCompressedData: ((filename:string) -> (CompressedImageData))&((fileData:FileData) -> (CompressedImageData)),
		newImageData: ((width:number,height:number) -> (ImageData))&((width:number,height:number,format:PixelFormat,data:string) -> (ImageData))&((width:number,height:number,data:string) -> (ImageData))&((filename:string) -> (ImageData))&((filedata:FileData) -> (ImageData)),
	},
} & {
	joystick: {
		getGamepadMappingString: ((guid:string) -> (string)),
		getJoystickCount: (() -> (number)),
		getJoysticks: (() -> ({Joystick})),
		loadGamepadMappings: ((filename:string) -> ())&((mappings:string) -> ()),
		saveGamepadMappings: ((filename:string) -> (string))&(() -> (string)),
		setGamepadMapping: ((guid:string,button:GamepadButton,inputtype:JoystickInputType,inputindex:number,hatdir:JoystickHat) -> (boolean))&((guid:string,axis:GamepadAxis,inputtype:JoystickInputType,inputindex:number,hatdir:JoystickHat) -> (boolean)),
	},
} & {
	keyboard: {
		getKeyFromScancode: ((scancode:Scancode) -> (KeyConstant)),
		getScancodeFromKey: ((key:KeyConstant) -> (Scancode)),
		hasKeyRepeat: (() -> (boolean)),
		hasScreenKeyboard: (() -> (boolean)),
		hasTextInput: (() -> (boolean)),
		isDown: ((key:KeyConstant) -> (boolean))&((key:KeyConstant,...KeyConstant) -> (boolean)),
		isScancodeDown: ((scancode:Scancode,...Scancode) -> (boolean)),
		setKeyRepeat: ((enable:boolean) -> ()),
		setTextInput: ((enable:boolean) -> ())&((enable:boolean,x:number,y:number,w:number,h:number) -> ()),
	},
} & {
	math: {
		colorFromBytes: ((rb:number,gb:number,bb:number,ab:number) -> (number,number,number,number)),
		colorToBytes: ((r:number,g:number,b:number,a:number) -> (number,number,number,number)),
		gammaToLinear: ((r:number,g:number,b:number) -> (number,number,number))&((color:{[any]: any}) -> (number,number,number))&((c:number) -> (number)),
		getRandomSeed: (() -> (number,number)),
		getRandomState: (() -> (string)),
		isConvex: ((vertices:{number}) -> (boolean))&((x1:number,y1:number,x2:number,y2:number,...number) -> (boolean)),
		linearToGamma: ((lr:number,lg:number,lb:number) -> (number,number,number))&((color:{number}) -> (number,number,number))&((lc:number) -> (number)),
		newBezierCurve: ((vertices:{number}) -> (BezierCurve))&((x1:number,y1:number,x2:number,y2:number,...number) -> (BezierCurve)),
		newRandomGenerator: (() -> (RandomGenerator))&((seed:number) -> (RandomGenerator))&((low:number,high:number) -> (RandomGenerator)),
		newTransform: (() -> (Transform))&((x:number,y:number,angle:number,sx:number,sy:number,ox:number,oy:number,kx:number,ky:number) -> (Transform)),
		noise: ((x:number) -> (number))&((x:number,y:number) -> (number))&((x:number,y:number,z:number) -> (number))&((x:number,y:number,z:number,w:number) -> (number)),
		random: (() -> (number))&((max:number) -> (number))&((min:number,max:number) -> (number)),
		randomNormal: ((stddev:number,mean:number) -> (number)),
		setRandomSeed: ((seed:number) -> ())&((low:number,high:number) -> ()),
		setRandomState: ((state:string) -> ()),
		triangulate: ((polygon:{[any]: any}) -> ({[any]: any}))&((x1:number,y1:number,x2:number,y2:number,x3:number,y3:number) -> ({[any]: any})),
	},
} & {
	mouse: {
		getCursor: (() -> (Cursor)),
		getPosition: (() -> (number,number)),
		getRelativeMode: (() -> (boolean)),
		getSystemCursor: ((ctype:CursorType) -> (Cursor)),
		getX: (() -> (number)),
		getY: (() -> (number)),
		isCursorSupported: (() -> (boolean)),
		isDown: ((button:number,...number) -> (boolean)),
		isGrabbed: (() -> (boolean)),
		isVisible: (() -> (boolean)),
		newCursor: ((imageData:ImageData,hotx:number,hoty:number) -> (Cursor))&((filename:string,hotx:number,hoty:number) -> (Cursor))&((fileData:FileData,hotx:number,hoty:number) -> (Cursor)),
		setCursor: ((cursor:Cursor) -> ())&(() -> ()),
		setGrabbed: ((grab:boolean) -> ()),
		setPosition: ((x:number,y:number) -> ()),
		setRelativeMode: ((enable:boolean) -> ()),
		setVisible: ((visible:boolean) -> ()),
		setX: ((x:number) -> ()),
		setY: ((y:number) -> ()),
	},
} & {
	physics: {
		getDistance: ((fixture1:Fixture,fixture2:Fixture) -> (number,number,number,number,number)),
		getMeter: (() -> (number)),
		newBody: ((world:World,x:number,y:number,type:BodyType) -> (Body)),
		newChainShape: ((loop:boolean,x1:number,y1:number,x2:number,y2:number,...number) -> (ChainShape))&((loop:boolean,points:{number}) -> (ChainShape)),
		newCircleShape: ((radius:number) -> (CircleShape))&((x:number,y:number,radius:number) -> (CircleShape)),
		newDistanceJoint: ((body1:Body,body2:Body,x1:number,y1:number,x2:number,y2:number,collideConnected:boolean) -> (DistanceJoint)),
		newEdgeShape: ((x1:number,y1:number,x2:number,y2:number) -> (EdgeShape)),
		newFixture: ((body:Body,shape:Shape,density:number) -> (Fixture)),
		newFrictionJoint: ((body1:Body,body2:Body,x:number,y:number,collideConnected:boolean) -> (FrictionJoint))&((body1:Body,body2:Body,x1:number,y1:number,x2:number,y2:number,collideConnected:boolean) -> (FrictionJoint)),
		newGearJoint: ((joint1:Joint,joint2:Joint,ratio:number,collideConnected:boolean) -> (GearJoint)),
		newMotorJoint: ((body1:Body,body2:Body,correctionFactor:number) -> (MotorJoint))&((body1:Body,body2:Body,correctionFactor:number,collideConnected:boolean) -> (MotorJoint)),
		newMouseJoint: ((body:Body,x:number,y:number) -> (MouseJoint)),
		newPolygonShape: ((x1:number,y1:number,x2:number,y2:number,x3:number,y3:number,...number) -> (PolygonShape))&((vertices:{number}) -> (PolygonShape)),
		newPrismaticJoint: ((body1:Body,body2:Body,x:number,y:number,ax:number,ay:number,collideConnected:boolean) -> (PrismaticJoint))&((body1:Body,body2:Body,x1:number,y1:number,x2:number,y2:number,ax:number,ay:number,collideConnected:boolean) -> (PrismaticJoint))&((body1:Body,body2:Body,x1:number,y1:number,x2:number,y2:number,ax:number,ay:number,collideConnected:boolean,referenceAngle:number) -> (PrismaticJoint)),
		newPulleyJoint: ((body1:Body,body2:Body,gx1:number,gy1:number,gx2:number,gy2:number,x1:number,y1:number,x2:number,y2:number,ratio:number,collideConnected:boolean) -> (PulleyJoint)),
		newRectangleShape: ((width:number,height:number) -> (PolygonShape))&((x:number,y:number,width:number,height:number,angle:number) -> (PolygonShape)),
		newRevoluteJoint: ((body1:Body,body2:Body,x:number,y:number,collideConnected:boolean) -> (RevoluteJoint))&((body1:Body,body2:Body,x1:number,y1:number,x2:number,y2:number,collideConnected:boolean,referenceAngle:number) -> (RevoluteJoint)),
		newRopeJoint: ((body1:Body,body2:Body,x1:number,y1:number,x2:number,y2:number,maxLength:number,collideConnected:boolean) -> (RopeJoint)),
		newWeldJoint: ((body1:Body,body2:Body,x:number,y:number,collideConnected:boolean) -> (WeldJoint))&((body1:Body,body2:Body,x1:number,y1:number,x2:number,y2:number,collideConnected:boolean) -> (WeldJoint))&((body1:Body,body2:Body,x1:number,y1:number,x2:number,y2:number,collideConnected:boolean,referenceAngle:number) -> (WeldJoint)),
		newWheelJoint: ((body1:Body,body2:Body,x:number,y:number,ax:number,ay:number,collideConnected:boolean) -> (WheelJoint))&((body1:Body,body2:Body,x1:number,y1:number,x2:number,y2:number,ax:number,ay:number,collideConnected:boolean) -> (WheelJoint)),
		newWorld: ((xg:number,yg:number,sleep:boolean) -> (World)),
		setMeter: ((scale:number) -> ()),
	},
} & {
	sound: {
		newDecoder: ((file:File,buffer:number) -> (Decoder))&((filename:string,buffer:number) -> (Decoder)),
		newSoundData: ((filename:string) -> (SoundData))&((file:File) -> (SoundData))&((decoder:Decoder) -> (SoundData))&((samples:number,rate:number,bits:number,channels:number) -> (SoundData)),
	},
} & {
	system: {
		getClipboardText: (() -> (string)),
		getOS: (() -> (string)),
		getPowerInfo: (() -> (PowerState,number,number)),
		getProcessorCount: (() -> (number)),
		hasBackgroundMusic: (() -> (boolean)),
		openURL: ((url:string) -> (boolean)),
		setClipboardText: ((text:string) -> ()),
		vibrate: ((seconds:number) -> ()),
	},
} & {
	thread: {
		getChannel: ((name:string) -> (Channel)),
		newChannel: (() -> (Channel)),
		newThread: ((filename:string) -> (Thread))&((fileData:FileData) -> (Thread))&((codestring:string) -> (Thread)),
	},
} & {
	timer: {
		getAverageDelta: (() -> (number)),
		getDelta: (() -> (number)),
		getFPS: (() -> (number)),
		getTime: (() -> (number)),
		sleep: ((s:number) -> ()),
		step: (() -> (number)),
	},
} & {
	touch: {
		getPosition: ((id:userdata) -> (number,number)),
		getPressure: ((id:userdata) -> (number)),
		getTouches: (() -> ({userdata})),
	},
} & {
	video: {
		newVideoStream: ((filename:string) -> (VideoStream))&((file:File) -> (VideoStream)),
	},
} & {
	window: {
		close: (() -> ()),
		fromPixels: ((pixelvalue:number) -> (number))&((px:number,py:number) -> (number,number)),
		getDPIScale: (() -> (number)),
		getDesktopDimensions: ((displayindex:number) -> (number,number)),
		getDisplayCount: (() -> (number)),
		getDisplayName: ((displayindex:number) -> (string)),
		getDisplayOrientation: ((displayindex:number) -> (DisplayOrientation)),
		getFullscreen: (() -> (boolean,FullscreenType)),
		getFullscreenModes: ((displayindex:number) -> ({width:number,height:number,})),
		getIcon: (() -> (ImageData)),
		getMode: (() -> (number,number,{fullscreen:boolean,fullscreentype:FullscreenType,vsync:boolean,msaa:number,resizable:boolean,borderless:boolean,centered:boolean,display:number,minwidth:number,minheight:number,highdpi:boolean,refreshrate:number,x:number,y:number,srgb:boolean,})),
		getPosition: (() -> (number,number,number)),
		getSafeArea: (() -> (number,number,number,number)),
		getTitle: (() -> (string)),
		getVSync: (() -> (number)),
		hasFocus: (() -> (boolean)),
		hasMouseFocus: (() -> (boolean)),
		isDisplaySleepEnabled: (() -> (boolean)),
		isMaximized: (() -> (boolean)),
		isMinimized: (() -> (boolean)),
		isOpen: (() -> (boolean)),
		isVisible: (() -> (boolean)),
		maximize: (() -> ()),
		minimize: (() -> ()),
		requestAttention: ((continuous:boolean) -> ()),
		restore: (() -> ()),
		setDisplaySleepEnabled: ((enable:boolean) -> ()),
		setFullscreen: ((fullscreen:boolean) -> (boolean))&((fullscreen:boolean,fstype:FullscreenType) -> (boolean)),
		setIcon: ((imagedata:ImageData) -> (boolean)),
		setMode: ((width:number,height:number,flags:{fullscreen:boolean?,fullscreentype:FullscreenType?,vsync:boolean?,msaa:number?,stencil:boolean?,depth:number?,resizable:boolean?,borderless:boolean?,centered:boolean?,display:number?,minwidth:number?,minheight:number?,highdpi:boolean?,x:number?,y:number?,usedpiscale:boolean?,srgb:boolean?,}) -> (boolean)),
		setPosition: ((x:number,y:number,displayindex:number) -> ()),
		setTitle: ((title:string) -> ()),
		setVSync: ((vsync:number) -> ()),
		showMessageBox: ((title:string,message:string,type:MessageBoxType,attachtowindow:boolean) -> (boolean))&((title:string,message:string,buttonlist:{[any]: any},type:MessageBoxType,attachtowindow:boolean) -> (number)),
		toPixels: ((value:number) -> (number))&((x:number,y:number) -> (number,number)),
		updateMode: ((width:number,height:number,settings:{fullscreen:boolean,fullscreentype:FullscreenType,vsync:boolean,msaa:number,resizable:boolean,borderless:boolean,centered:boolean,display:number,minwidth:number,minheight:number,highdpi:boolean,x:number,y:number,}) -> (boolean)),
	}
}


type IOFile = {
    close: (self: IOFile) -> boolean,
    flush: (self: IOFile) -> boolean,
    lines: (self: IOFile, format: string?) -> (() -> string?)?,
    read: (self: IOFile, ...string) -> ...any,
    seek: (self: IOFile, whence: string?, offset: number?) -> number,
    setvbuf: (self: IOFile, mode: string, size: number?) -> boolean,
    write: (self: IOFile, ...string) -> IOFile,
}

declare io: {
    close: (file: IOFile?) -> boolean,
    flush: () -> boolean,
    input: (file: (string | IOFile)?) -> IOFile,
    lines: (filename: string?, format: string?) -> (() -> string?)?,
    open: (filename: string, mode: string?) -> IOFile?,
    output: (file: (string | IOFile)?) -> IOFile,
    popen: (prog: string, mode: string) -> IOFile,
    read: (...string) -> ...any,
    tmpfile: () -> IOFile,
    type: (obj: any) -> ("file" | "closed file" | nil),
    write: (...string) -> IOFile,
}

export type EnetEvent = {
    type:"receive" | "disconnect" | "connect",
    peer:Peer?,
    data:string | number,
    channel:number?
}

export type Peer = {
    disconnect:(self:Peer, data:number) -> nil;
    disconnect_now:(self:Peer, data:number) -> nil;
    disconnect_later:(self:Peer, data:number) -> nil;
    reset:(self:Peer) -> nil;
    ping:(self:Peer) -> nil;
    receive:(self:Peer) -> (string, number),
    send:(self:Peer, data:string, channel:number, flag:"reliable" | "unsequenced", "unreliable") -> nil,
    ping_interval:(self:Peer, interval:number) -> nil;
    timeout:(self:Peer, limit:number, minimum:number, maximum:number) -> nil;
    index:(self:Peer) -> number;
    state:(self:Peer) -> "disconnected" | "connecting" | "acknowledging_connect" | "connection_pending" | "connection_succeeded" | "connected" | "disconnect_later" | "disconnecting" | "acknowledging_disconnect" | "zombie" | "unknown";
    connect_id:(self:Peer) -> number;
    round_trip_time:(self:Peer) -> number;
    last_round_trip_time:(self:Peer) -> number
}

export type Enet_Host = {
    service:(self:Peer, timeout:number) -> EnetEvent,
    check_events:(self:Peer) -> {EnetEvent},
    connect:(self:Peer, address:string, channel_count:number, data:number) -> Peer,
    flush:(self:Peer) -> nil,
    broadcast:(self:Peer, data:string, channel:number, flag:"reliable" | "unsequenced", "unreliable") -> nil,
    channel_limit:(self:Peer, limit:number) -> nil,
    bandwidth_limit:(self:Peer, incoming:number, outgoing:number) -> nil,
    get_socket_address:(self:Peer) -> string,
    destroy:(self:Peer) -> nil,
    total_sent_data:(self:Peer) -> number,
    total_received_data:(self:Peer) -> number,
    peer_count:(self:Peer) -> number,
    get_peer:(self:Peer, index:number) -> Peer
}

declare Enet:{
	host_create:(bind_address:string, peer_count:number, channel_count:number, in_bandwidth:number, out_bandwidth:number) -> Enet_Host
}

export type SocketError = "closed" | "timeout" | "refused" | "Address already in use"

export type TimeoutValue = number | "b" | nil

export type Socket = {
	close: (self: Socket) -> (number | nil, SocketError?),
	getfd: (self: Socket) -> number,
	getstats: (self: Socket) -> (number, number, number),
	setstats: (self: Socket, received: number, sent: number, age: number) -> (number),
	settimeout: (self: Socket, value: TimeoutValue, mode: ("b" | "t")?) -> (),
	gettimeout: (self: Socket) -> TimeoutValue,
	dirty: (self: Socket) -> boolean,
}

export type TCPSocket = Socket & {
	accept: (self: TCPSocket) -> (TCPSocket?, SocketError?),
	bind: (self: TCPSocket, address: string, port: number) -> (number | nil, SocketError?),
	connect: (self: TCPSocket, address: string, port: number, locaddr: string?, locport: number?) -> (number | nil, SocketError?),
	getpeername: (self: TCPSocket) -> (string?, number?),
	getsockname: (self: TCPSocket) -> (string?, number?),
	listen: (self: TCPSocket, backlog: number?) -> (number | nil, SocketError?),
	receive: (self: TCPSocket, pattern: (number | "*a" | "*l" | "*L")?, prefix: string?) -> (string?, SocketError?, string?),
	send: (self: TCPSocket, data: string, i: number?, j: number?) -> (number | nil, SocketError?, number?),
	setoption: (self: TCPSocket, option: string, value: any) -> (number | nil, SocketError?),
	getoption: (self: TCPSocket, option: string) -> (any, SocketError?),
	shutdown: (self: TCPSocket, mode: ("both" | "send" | "receive")?) -> (number | nil, SocketError?),
}


export type UDPSocket = Socket & {
	receivefrom: (self: UDPSocket, size: number?) -> (string?, string?, number?, SocketError?),
	sendto: (self: UDPSocket, datagram: string, ip: string, port: number) -> (number | nil, SocketError?),
	setpeername: (self: UDPSocket, address: string, port: number) -> (number | nil, SocketError?),
	setsockname: (self: UDPSocket, address: string, port: number) -> (number | nil, SocketError?),
	getpeername: (self: UDPSocket) -> (string?, number?),
	getsockname: (self: UDPSocket) -> (string?, number?),
	setoption: (self: UDPSocket, option: string, value: any) -> (number | nil, SocketError?),
	getoption: (self: UDPSocket, option: string) -> (any, SocketError?),
}

export type UnixSocket = Socket & {
	accept: (self: UnixSocket) -> (UnixSocket?, SocketError?),
	bind: (self: UnixSocket, path: string) -> (number | nil, SocketError?),
	connect: (self: UnixSocket, path: string) -> (number | nil, SocketError?),
	listen: (self: UnixSocket, backlog: number?) -> (number | nil, SocketError?),
	receive: (self: UnixSocket, pattern: (number | "*a" | "*l" | "*L")?, prefix: string?) -> (string?, SocketError?, string?),
	send: (self: UnixSocket, data: string, i: number?, j: number?) -> (number | nil, SocketError?, number?),
	shutdown: (self: UnixSocket, mode: ("both" | "send" | "receive")?) -> (number | nil, SocketError?),
}

export type HTTPResult = {
	body: string,
	code: number,
	headers: {[string]: string},
	status: string,
}

export type HTTPRequest = {
	url: string,
	sink: any?,
	method: string?,
	headers: {[string]: string}?,
	source: any?,
	step: any?,
	proxy: string?,
	redirect: boolean?,
	create: () -> TCPSocket?,
}

export type FTPResult = {
	body: string?,
	code: number,
	message: string,
}

export type SMTPMessage = {
	headers: {[string]: string},
	body: string | {string},
}

export type SMTPSource = () -> (string?)

export type SMTPServer = {
	server: string,
	port: number?,
	domain: string?,
	user: string?,
	password: string?,
	create: (() -> TCPSocket)?,
}

export type URLComponents = {
	scheme: string?,
	authority: string?,
	userinfo: string?,
	user: string?,
	password: string?,
	host: string?,
	port: number?,
	path: string?,
	params: string?,
	query: string?,
	fragment: string?,
}

export type MimeSource = () -> (string?)
export type MimeSink = (chunk: string?, err: string?) -> ()

export type LTN12Source = () -> (string?, string?)
export type LTN12Sink = (chunk: string?, err: string?) -> (string?)
export type LTN12Filter = (chunk: string?) -> (string?)
export type LTN12Pump = {
	step: (src: LTN12Source, snk: LTN12Sink) -> (string?),
	all: (src: LTN12Source, snk: LTN12Sink) -> (number | nil, string?),
}

declare Socket:{
	_VERSION: string,
	_DEBUG: boolean,
	
	connect: (address: string, port: number, locaddr: string?, locport: number?) -> (TCPSocket | nil, SocketError?),
	bind: (host: string, port: number, backlog: number?) -> (TCPSocket | nil, SocketError?),
	tcp: () -> TCPSocket,
	udp: () -> UDPSocket,
	unix: () -> UnixSocket?,
	
	choose: (...any) -> (...any),
	gettime: () -> number,
	newtry: (finalizer: ((...any) -> ())?) -> ((...any) -> (...any)),
	protect: (func: (...any) -> (...any)) -> ((...any) -> (boolean, ...any)),
	select: (recvt: {Socket}, sendt: {Socket}?, timeout: number?) -> ({Socket}, {Socket}, SocketError?),
	sink: (mode: string, socket: Socket) -> LTN12Sink,
	skip: (d: number, ret: (...any) -> (...any)) -> (...any) -> (...any),
	sleep: (time: number) -> (),
	source: (mode: string, socket: Socket, length: number?) -> LTN12Source,
	
	dns: {
		toip: (address: string) -> (string | nil, string?),
		tohostname: (address: string) -> (string | nil, string?),
		gethostname: () -> (string | nil, string?),
	},
	
	http: {
		request: ((HTTPRequest | string), body: string?) -> (HTTPResult | string | nil, number?, {[string]: string}?, string?),
	},
	
	ftp: {
		get: (url: string | {url: string, sink: any, type: ("i" | "a")?}) -> (FTPResult | nil, string?),
		put: (url: string | {url: string, source: any, type: ("i" | "a")?}, body: string?) -> (FTPResult | nil, string?),
	},
	
	smtp: {
		message: (mesgt: SMTPMessage) -> SMTPSource,
		send: (mail: {
			from: string,
			rcpt: string | {string},
			source: SMTPSource,
			server: string?,
			port: number?,
			domain: string?,
			user: string?,
			password: string?,
		}) -> (number | nil, string?),
	},
	
	url: {
		build: (components: URLComponents) -> string,
		build_path: (segments: {string}, unsafe: boolean?) -> string,
		parse: (url: string, default: URLComponents?) -> URLComponents,
		parse_path: (path: string) -> {string},
		parse_query: (query: string, sep: string?) -> {[string]: string},
		absolute: (base: string, relative: string) -> string,
		escape: (s: string) -> string,
		unescape: (s: string) -> string,
	},
	
	mime: {
		b64: (data: string) -> string,
		unb64: (data: string) -> string,
		qp: (data: string, mode: ("text" | "binary")?) -> string,
		unqp: (data: string) -> string,
		wrp: (length: number, data: string, indent: number?) -> string,
		qpwrp: (data: string, length: number?, indent: number?) -> string,
		eol: (mode: number, data: string) -> string,
		dot: (mode: number, data: string) -> string,
	},
	
	ltn12: {
		BLOCKSIZE: number,
		
		source: {
			cat: (...LTN12Source) -> LTN12Source,
			chain: (src: LTN12Source, filter: LTN12Filter) -> LTN12Source,
			empty: () -> LTN12Source,
			error: (err: string) -> LTN12Source,
			file: (file: any, message: string?) -> LTN12Source,
			simplify: (src: LTN12Source) -> LTN12Source,
			string: (s: string) -> LTN12Source,
		},
		
		filter: {
			chain: (...LTN12Filter) -> LTN12Filter,
			cycle: (low: LTN12Filter, ctx: any, extra: any) -> LTN12Filter,
		},
		
		sink: {
			chain: (filter: LTN12Filter, sink: LTN12Sink) -> LTN12Sink,
			error: (err: string) -> LTN12Sink,
			file: (file: any, message: string?) -> LTN12Sink,
			null: () -> LTN12Sink,
			simplify: (sink: LTN12Sink) -> LTN12Sink,
			table: (t: {string}?) -> LTN12Sink,
		},
		
		pump: LTN12Pump,
	},
}

declare os: typeof(os) & {
	execute: (command: string) -> (number | boolean | nil);
	exit: (code: number?, close: boolean?) -> never;
	getenv: (varname: string) -> string | nil;
	remove: (filename: string) -> boolean;
	rename: (oldname: string, newname: string) -> boolean;
	setlocale: (locale: string?, category: string?) -> string | nil;
	tmpname: () -> string;
}

declare package: {
    path: string,
    cpath: string,
    loaded: { [string]: any },
    preload: { [string]: () -> any },
    searchers: { (modname: string) -> (() -> any) | string },

    loadlib: (libname: string, funcname: string) -> (() -> any)?,
}

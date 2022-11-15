#include "PIDefines.h"
#include "PIResourceDefines.h"

#ifdef __PIMac__
    #include <Carbon.r>
	#include "PIGeneral.r"
	#include "PIUtilities.r"
#elif defined(__PIWin__)
	#define Rez
	#include "PIGeneral.h"
	#include "PIUtilities.r"
#endif

#include "PIActions.h"


#define plugInSuiteID		'mgai'
#define plugInClassID		plugInSuiteID
#define plugInEventID		plugInClassID
#define plugInUniqueID		"7eb8ac17-4a2e-4c9f-bfb4-aa41f930148b"

resource 'PiPL' ( 16000, "MangAI", purgeable )
{
	{
		Kind { Filter },
		Name { "Clean Manga Page..." },
		Category { "MangAI" },
		Version { (latestFilterVersion << 16 ) | latestFilterSubVersion },

		Component { ComponentNumber, plugInName },

		#if __PIMac__
			#if defined(__arm64__)
				CodeMacARM64 { "PluginMain" },
			#endif
			#if defined(__x86_64__)
				CodeMacIntel64 { "PluginMain" },
			#endif
		#elif __PIWin__
			CodeEntryPointWin64 { "PluginMain" },
		#endif

		SupportedModes
		{
			noBitmap, doesSupportGrayScale,
			noIndexedColor, doesSupportRGBColor,
			doesSupportCMYKColor, doesSupportHSLColor,
			doesSupportHSBColor, doesSupportMultichannel,
			doesSupportDuotone, doesSupportLABColor
		},

		HasTerminology
		{
			plugInClassID,
			plugInEventID,
			16000,
			plugInUniqueID
		},

		EnableInfo { "in (PSHOP_ImageMode, RGBMode, GrayScaleMode,"
		             "CMYKMode, HSLMode, HSBMode, MultichannelMode,"
					 "DuotoneMode, LabMode, RGB48Mode, Gray16Mode) ||"
					 "PSHOP_ImageDepth == 16 ||"
					 "PSHOP_ImageDepth == 32" },

		PlugInMaxSize { 2000000, 2000000 },

		MonitorScalingAware {},

		FilterLayerSupport {doesSupportFilterLayers},

		FilterCaseInfo
		{
			{
				/* Flat data, no selection */
				inWhiteMat, outWhiteMat,
				doNotWriteOutsideSelection,
				filtersLayerMasks, worksWithBlankData,
				copySourceToDestination,

				/* Flat data with selection */
				inWhiteMat, outWhiteMat,
				writeOutsideSelection,
				filtersLayerMasks, worksWithBlankData,
				copySourceToDestination,

				/* Floating selection */
				inWhiteMat, outWhiteMat,
				writeOutsideSelection,
				filtersLayerMasks, worksWithBlankData,
				copySourceToDestination,

				/* Editable transparency, no selection */
				inWhiteMat, outWhiteMat,
				doNotWriteOutsideSelection,
				filtersLayerMasks, worksWithBlankData,
				copySourceToDestination,

				/* Editable transparency, with selection */
				inWhiteMat, outWhiteMat,
				writeOutsideSelection,
				filtersLayerMasks, worksWithBlankData,
				copySourceToDestination,

				/* Preserved transparency, no selection */
				inWhiteMat, outWhiteMat,
				doNotWriteOutsideSelection,
				filtersLayerMasks, worksWithBlankData,
				copySourceToDestination,

				/* Preserved transparency, with selection */
				inWhiteMat, outWhiteMat,
				writeOutsideSelection,
				filtersLayerMasks, worksWithBlankData,
				copySourceToDestination
			}
		}
	}
};
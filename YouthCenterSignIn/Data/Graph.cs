using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.Http;
using System.Text;
using System.Threading.Tasks;
using Microsoft.Graph;
using Microsoft.Toolkit.Services.MicrosoftGraph;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn
{
    class Graph
    {
        const string AppId = "6cd2d40f-e8d2-46be-ad3f-ac087d4b5284";
        readonly string[] Scopes = { "User.Read", "Contacts.ReadWrite" };

        public Graph()
        {
            MicrosoftGraphService instance = MicrosoftGraphService.Instance;
            instance.AuthenticationModel = MicrosoftGraphEnums.AuthenticationModel.V2;
            instance.Initialize(AppId,
                MicrosoftGraphEnums.ServicesToInitialize.UserProfile,
                Scopes);
        }

        public bool IsAuthenticated => MicrosoftGraphService.Instance.IsAuthenticated;

        public GraphServiceClient Provider => MicrosoftGraphService.Instance.GraphProvider;

        public async Task<bool> Login() => await MicrosoftGraphService.Instance.TryLoginAsync();

        #region Contacts

        public async Task<IEnumerable<Contact>> GetContacts()
        {
            if (!IsAuthenticated)
            {
                await ((UwpDataProvider)DataProvider.Current).ShowAuthenticationMessage();
                return new List<Contact>();
            }

            var contactFolder = await GetContactFolder();
            return contactFolder.Contacts;
        }

        public async Task<Contact> SaveContact(Contact contact)
        {
            try
            {
                if (!IsAuthenticated)
                    throw new InvalidOperationException("You must be logged in to save a contact!");

                var folderId = cachedFolderId ?? (await GetContactFolder(getContacts: false)).Id;

                var request = GetNewContactRequest(contact, folderId);
                var response = await SendRequest(request);

                var contactJson = await response.Content.ReadAsStringAsync();
                var newContact = Provider.HttpProvider.Serializer.DeserializeObject<Contact>(contactJson);

                return newContact;
            }
            catch (Exception ex)
            {
                throw new Exception($"Contact save failed!", ex);
            }
        }

        HttpRequestMessage GetNewContactRequest(Contact contact, string folderId)
        {
            var requestUrl = Provider.Me.ContactFolders.Request().RequestUrl;
            requestUrl += $"/{folderId}/contacts";

            var request = new HttpRequestMessage(HttpMethod.Post, requestUrl)
            {
                Content = new StringContent(ContactToJson(contact), Encoding.UTF8, "application/json")
            };
            return request;
        }

        string ContactToJson(Contact contact) => Provider.HttpProvider.Serializer.SerializeObject(contact);

        async Task<HttpResponseMessage> SendRequest(HttpRequestMessage request)
        {
            await Provider.AuthenticationProvider.AuthenticateRequestAsync(request);
            var response = await Provider.HttpProvider.SendAsync(request);
            if (!response.IsSuccessStatusCode)
            {
                throw new ServiceException(new Error
                {
                    Code = response.StatusCode.ToString(),
                    Message = await response.Content.ReadAsStringAsync()
                });
            }

            return response;
        }

        #endregion

        #region Contact folders

        const string YouthCenterFolderName = "Youth Center";

        string cachedFolderId;
        async Task<ContactFolder> GetContactFolder(bool getContacts = true)
        {
            var folder = await DownloadContactFolder(getContacts) ?? await CreateContactFolder();
            cachedFolderId = folder.Id;
            return folder;
        }

        async Task<ContactFolder> DownloadContactFolder(bool getContacts)
        {
            var folders = await GetContactFolderQuery(getContacts).GetAsync();
            return folders.FirstOrDefault();
        }

        IUserContactFoldersCollectionRequest GetContactFolderQuery(bool getContacts)
        {
            var query = Provider.Me.ContactFolders.Request()
                .Filter($"displayName eq '{YouthCenterFolderName}'")
                .Select("id")
                .Top(1);

            if (getContacts)
                query = query.Expand("contacts($select=id,givenName,surname,birthday,homeAddress)");

            return query;
        }

        async Task<ContactFolder> CreateContactFolder()
        {
            var newFolder = new ContactFolder { DisplayName = YouthCenterFolderName };
            var folder = await Provider.Me.ContactFolders.Request()
                .Select("id")
                .AddAsync(newFolder);
            return folder;
        }

        #endregion
    }
}
